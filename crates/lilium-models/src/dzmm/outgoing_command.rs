use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OutgoingCommand {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub account_user_id: String,
    pub event: String,
    pub data: serde_json::Value,
    pub require_ack: bool,
    pub status: String,
    pub processed_at: Option<DateTime<Utc>>,
    pub ack_response: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub attempt_count: i32,
    pub max_attempts: i32,
}

pub mod status {
    pub const PENDING: &str = "pending";
    pub const PROCESSING: &str = "processing";
    pub const SUCCESS: &str = "success";
    pub const FAILED: &str = "failed";
    pub const TIMEOUT: &str = "timeout";

    pub const TERMINAL_STATUSES: &[&str] = &[SUCCESS, FAILED, TIMEOUT];
}
