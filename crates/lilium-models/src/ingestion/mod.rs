use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebSocketEvent {
    pub id: Option<i64>,
    pub event: String,
    pub data: serde_json::Value,
    pub user_id: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub account_user_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub received_at: DateTime<Utc>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventProcessorOffset {
    pub processor_id: String,
    pub last_processed_id: i64,
    pub last_processed_timestamp: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}
