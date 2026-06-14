use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub user_id: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub avatar_file: Option<String>,
    pub bio: Option<String>,
    pub birthday: Option<String>,
    pub birthday_public: Option<bool>,
    pub quirk: Option<String>,
    pub is_bot: Option<bool>,
    pub gender: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub raw_data: Option<serde_json::Value>,
    pub last_seen: Option<DateTime<Utc>>,
    pub message_count: i32,
    pub deleted_count: i32,
    pub recalled_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
