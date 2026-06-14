use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiClientConfig {
    pub base_url: String,
    pub ws_url: String,
    pub request_timeout_secs: u64,
    pub reconnect_delay_ms: u64,
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        Self {
            base_url: "https://dzmm.com".to_string(),
            ws_url: "wss://dzmm.com/ws".to_string(),
            request_timeout_secs: 30,
            reconnect_delay_ms: 5000,
        }
    }
}
