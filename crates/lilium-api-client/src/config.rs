use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiClientConfig {
    pub base_url: String,
    pub ws_url: String,
    pub request_timeout_secs: u64,
    pub reconnect_delay_ms: u64,
}

pub const DZMM_BASE_URL: &str = "https://www.dzmm.ai";
pub const DZMM_SOCKETIO_URL: &str = "https://www.dzmm.ai/ws/matching/";

impl Default for ApiClientConfig {
    fn default() -> Self {
        Self {
            base_url: DZMM_BASE_URL.to_string(),
            ws_url: DZMM_SOCKETIO_URL.to_string(),
            request_timeout_secs: 30,
            reconnect_delay_ms: 5000,
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
