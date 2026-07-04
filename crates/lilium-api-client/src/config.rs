// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 dzmm_client/api.py

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiClientConfig {
    pub base_url: String,
    pub ws_url: String,
    pub request_timeout_secs: u64,
    pub reconnect_delay_ms: u64,
    pub min_request_delay: f64,
    pub max_request_delay: f64,
    pub request_batch_size: u64,
    pub request_batch_delay: f64,
}

pub const DZMM_BASE_URL: &str = "https://www.dzmm.ai";
pub const DZMM_SOCKETIO_URL: &str = "https://www.dzmm.ai/ws/matching/";

fn env_f64(name: &str, default: f64) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        Self {
            base_url: DZMM_BASE_URL.to_string(),
            ws_url: DZMM_SOCKETIO_URL.to_string(),
            request_timeout_secs: 30,
            reconnect_delay_ms: 5000,
            // Rate-limit defaults; env vars preserve the previous override
            // semantics (the read moved here from RateLimiter::new).
            min_request_delay: env_f64("MIN_REQUEST_DELAY", 0.2),
            max_request_delay: env_f64("MAX_REQUEST_DELAY", 0.5),
            request_batch_size: env_u64("BATCH_SIZE", 50),
            request_batch_delay: env_f64("BATCH_DELAY", 1.0),
        }
    }
}

pub fn parse_dzmm_local_address(value: Option<&str>) -> anyhow::Result<Option<IpAddr>> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    if value.eq_ignore_ascii_case("auto") {
        return Ok(None);
    }

    match value {
        "0.0.0.0" | "::" => Ok(Some(value.parse()?)),
        _ => anyhow::bail!("DZMM_HTTP_LOCAL_ADDRESS must be 0.0.0.0, ::, auto, or unset"),
    }
}

pub fn dzmm_local_address_from_env() -> anyhow::Result<Option<IpAddr>> {
    parse_dzmm_local_address(std::env::var("DZMM_HTTP_LOCAL_ADDRESS").ok().as_deref())
}
