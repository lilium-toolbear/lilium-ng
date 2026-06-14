use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Room {
    pub room_id: String,
    pub title: String,
    pub chat_type: Option<String>,
    pub avatar_url: Option<String>,
    pub member_count: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub is_public: Option<bool>,
    pub creator_id: Option<String>,
    pub account_ids: Vec<String>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub first_message_at: Option<DateTime<Utc>>,
    pub backfill_until: Option<DateTime<Utc>>,
    pub history_complete: bool,
    pub message_count: i32,
    pub deleted_count: i32,
    pub recalled_count: i32,
    pub edited_count: i32,
    pub image_count: i32,
    pub is_active: bool,
    pub dissolved_at: Option<DateTime<Utc>>,
    pub raw_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
