use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use url::Url;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ClearanceCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: f64,
}

impl std::fmt::Debug for ClearanceCookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClearanceCookie")
            .field("name", &self.name)
            .field("value", &"<redacted>")
            .field("domain", &self.domain)
            .field("path", &self.path)
            .field("expires", &self.expires)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClearanceSnapshot {
    pub generation: u64,
    pub user_agent: String,
    pub cookies: Vec<ClearanceCookie>,
    pub expires_at: DateTime<Utc>,
    pub verified_at: DateTime<Utc>,
}

impl ClearanceSnapshot {
    pub fn validate_at(&self, now: DateTime<Utc>) -> Result<(), ClearanceError> {
        if self.generation == 0 {
            return Err(ClearanceError::InvalidSnapshot(
                "generation must be greater than zero".to_string(),
            ));
        }
        if self.user_agent.trim().is_empty() {
            return Err(ClearanceError::InvalidSnapshot(
                "user agent must not be empty".to_string(),
            ));
        }
        if self.expires_at <= now {
            return Err(ClearanceError::InvalidSnapshot(format!(
                "snapshot expired at {}",
                self.expires_at
            )));
        }
        let clearance_cookie = self
            .cookies
            .iter()
            .find(|cookie| cookie.name == "cf_clearance" && !cookie.value.is_empty())
            .ok_or_else(|| {
                ClearanceError::InvalidSnapshot("snapshot has no cf_clearance cookie".to_string())
            })?;
        if !clearance_cookie.expires.is_finite()
            || clearance_cookie.expires <= now.timestamp() as f64
        {
            return Err(ClearanceError::InvalidSnapshot(
                "cf_clearance cookie expired".to_string(),
            ));
        }
        if let Some(cookie) = self
            .cookies
            .iter()
            .find(|cookie| !is_cloudflare_cookie_name(&cookie.name))
        {
            return Err(ClearanceError::InvalidSnapshot(format!(
                "snapshot contains non-Cloudflare cookie {}",
                cookie.name
            )));
        }
        Ok(())
    }

    pub fn merge_cookie_header(&self, account_cookie_header: Option<&str>) -> String {
        let mut pairs = Vec::with_capacity(self.cookies.len() + 4);
        let mut cloudflare_names = HashSet::with_capacity(self.cookies.len());

        for cookie in &self.cookies {
            cloudflare_names.insert(cookie.name.as_str());
            pairs.push(format!("{}={}", cookie.name, cookie.value));
        }

        if let Some(account_cookie_header) = account_cookie_header {
            for pair in account_cookie_header.split(';').map(str::trim) {
                let Some((name, _)) = pair.split_once('=') else {
                    continue;
                };
                if !name.is_empty() && !cloudflare_names.contains(name) {
                    pairs.push(pair.to_string());
                }
            }
        }

        pairs.join("; ")
    }
}

pub(crate) fn is_cloudflare_cookie_name(name: &str) -> bool {
    name == "cf_clearance" || name.starts_with("__cf") || name.starts_with("_cf")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClearanceError {
    AgentUnavailable {
        operation: &'static str,
        status: Option<StatusCode>,
        code: Option<String>,
        message: String,
        retryable: bool,
    },
    InvalidConfiguration(String),
    InvalidSnapshot(String),
    ChallengePersisted {
        endpoint: String,
        generation: u64,
    },
}

impl ClearanceError {
    pub fn retryable(&self) -> bool {
        match self {
            Self::AgentUnavailable { retryable, .. } => *retryable,
            Self::InvalidConfiguration(_) => false,
            Self::InvalidSnapshot(_) | Self::ChallengePersisted { .. } => true,
        }
    }
}

impl std::fmt::Display for ClearanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AgentUnavailable {
                operation,
                status,
                code,
                message,
                ..
            } => {
                let code = code
                    .as_deref()
                    .map(|code| format!(" [{code}]"))
                    .unwrap_or_default();
                if let Some(status) = status {
                    write!(
                        f,
                        "Cloudflare clearance agent {operation} failed with {status}{code}: {message}"
                    )
                } else {
                    write!(
                        f,
                        "Cloudflare clearance agent {operation} failed{code}: {message}"
                    )
                }
            }
            Self::InvalidSnapshot(message) => {
                write!(f, "Invalid Cloudflare clearance snapshot: {message}")
            }
            Self::InvalidConfiguration(message) => {
                write!(f, "Invalid Cloudflare clearance configuration: {message}")
            }
            Self::ChallengePersisted {
                endpoint,
                generation,
            } => write!(
                f,
                "Cloudflare challenge persisted for {endpoint} after refreshing generation {generation}"
            ),
        }
    }
}

impl std::error::Error for ClearanceError {}

pub type ClearanceFuture<'a> =
    Pin<Box<dyn Future<Output = Result<ClearanceSnapshot, ClearanceError>> + Send + 'a>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClearanceRefreshReason {
    CfMitigated,
}

pub trait ClearanceProvider: Send + Sync {
    fn snapshot(&self) -> ClearanceFuture<'_>;
    fn refresh(
        &self,
        observed_generation: u64,
        reason: ClearanceRefreshReason,
    ) -> ClearanceFuture<'_>;
}

#[derive(Debug, Clone)]
pub struct ClearanceAgentClient {
    client: reqwest::Client,
    base_url: Url,
}

impl ClearanceAgentClient {
    pub fn new(base_url: &str) -> Result<Self, ClearanceError> {
        let normalized = format!("{}/", base_url.trim().trim_end_matches('/'));
        let base_url = Url::parse(&normalized).map_err(|error| {
            ClearanceError::InvalidConfiguration(format!("invalid clearance agent URL: {error}"))
        })?;
        if !matches!(base_url.scheme(), "http" | "https") {
            return Err(ClearanceError::InvalidConfiguration(
                "clearance agent URL must use http or https".to_string(),
            ));
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(100))
            .build()
            .map_err(|error| ClearanceError::AgentUnavailable {
                operation: "initialization",
                status: None,
                code: None,
                message: error.to_string(),
                retryable: false,
            })?;
        Ok(Self { client, base_url })
    }

    #[instrument(level = "debug", skip(self))]
    async fn get_snapshot(&self) -> Result<ClearanceSnapshot, ClearanceError> {
        let url = self.base_url.join("v1/snapshot").map_err(|error| {
            ClearanceError::InvalidConfiguration(format!("invalid snapshot endpoint: {error}"))
        })?;
        let response = self.client.get(url).send().await.map_err(|error| {
            ClearanceError::AgentUnavailable {
                operation: "snapshot",
                status: None,
                code: None,
                message: error.to_string(),
                retryable: true,
            }
        })?;
        self.parse_snapshot_response("snapshot", response).await
    }

    #[instrument(level = "debug", skip(self), fields(observed_generation, reason))]
    async fn refresh_snapshot(
        &self,
        observed_generation: u64,
        reason: ClearanceRefreshReason,
    ) -> Result<ClearanceSnapshot, ClearanceError> {
        let url = self.base_url.join("v1/refresh").map_err(|error| {
            ClearanceError::InvalidConfiguration(format!("invalid refresh endpoint: {error}"))
        })?;
        let response = self
            .client
            .post(url)
            .json(&RefreshRequest {
                observed_generation,
                reason,
            })
            .send()
            .await
            .map_err(|error| ClearanceError::AgentUnavailable {
                operation: "refresh",
                status: None,
                code: None,
                message: error.to_string(),
                retryable: true,
            })?;
        self.parse_snapshot_response("refresh", response).await
    }

    async fn parse_snapshot_response(
        &self,
        operation: &'static str,
        response: reqwest::Response,
    ) -> Result<ClearanceSnapshot, ClearanceError> {
        let status = response.status();
        let body = response
            .bytes()
            .await
            .map_err(|error| ClearanceError::AgentUnavailable {
                operation,
                status: Some(status),
                code: None,
                message: error.to_string(),
                retryable: true,
            })?;
        if !status.is_success() {
            let parsed_error = serde_json::from_slice::<AgentErrorResponse>(&body).ok();
            let code = parsed_error
                .as_ref()
                .map(|payload| payload.error.code.clone());
            let retryable = parsed_error
                .as_ref()
                .map(|payload| payload.error.retryable)
                .unwrap_or(status.is_server_error());
            let message = parsed_error
                .map(|payload| payload.error.message)
                .unwrap_or_else(|| String::from_utf8_lossy(&body).into_owned());
            return Err(ClearanceError::AgentUnavailable {
                operation,
                status: Some(status),
                code,
                message,
                retryable,
            });
        }

        let snapshot: ClearanceSnapshot = serde_json::from_slice(&body).map_err(|error| {
            ClearanceError::InvalidSnapshot(format!(
                "agent returned invalid {operation} response: {error}"
            ))
        })?;
        snapshot.validate_at(Utc::now())?;
        Ok(snapshot)
    }
}

impl ClearanceProvider for ClearanceAgentClient {
    fn snapshot(&self) -> ClearanceFuture<'_> {
        Box::pin(self.get_snapshot())
    }

    fn refresh(
        &self,
        observed_generation: u64,
        reason: ClearanceRefreshReason,
    ) -> ClearanceFuture<'_> {
        Box::pin(self.refresh_snapshot(observed_generation, reason))
    }
}

#[derive(Serialize)]
struct RefreshRequest {
    observed_generation: u64,
    reason: ClearanceRefreshReason,
}

#[derive(Deserialize)]
struct AgentErrorResponse {
    error: AgentErrorBody,
}

#[derive(Deserialize)]
struct AgentErrorBody {
    code: String,
    message: String,
    retryable: bool,
}

#[cfg(test)]
mod tests {
    use super::{ClearanceCookie, ClearanceSnapshot};
    use chrono::{Duration, Utc};

    #[test]
    fn snapshot_debug_output_redacts_cookie_values() {
        let secret = "sensitive-clearance-value";
        let snapshot = ClearanceSnapshot {
            generation: 7,
            user_agent: "browser-user-agent".to_string(),
            cookies: vec![ClearanceCookie {
                name: "cf_clearance".to_string(),
                value: secret.to_string(),
                domain: ".dzmm.ai".to_string(),
                path: "/".to_string(),
                expires: (Utc::now() + Duration::hours(1)).timestamp() as f64,
            }],
            expires_at: Utc::now() + Duration::hours(1),
            verified_at: Utc::now(),
        };

        let debug_output = format!("{snapshot:?}");

        assert!(!debug_output.contains(secret));
        assert!(debug_output.contains("<redacted>"));
    }
}
