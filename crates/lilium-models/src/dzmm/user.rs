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

impl User {
    pub fn from_api(data: &serde_json::Value) -> Option<Self> {
        let obj = data.as_object()?;
        let user_id = obj
            .get("id")
            .or_else(|| obj.get("userId"))
            .or_else(|| obj.get("user_id"))
            .and_then(|v| v.as_str())?
            .to_string();

        let now = Utc::now();
        let last_seen = obj
            .get("lastSeen")
            .or_else(|| obj.get("last_seen"))
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Some(Self {
            user_id,
            full_name: obj
                .get("fullName")
                .or_else(|| obj.get("full_name"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            avatar_url: obj
                .get("avatarUrl")
                .or_else(|| obj.get("avatar_url"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            avatar_file: None,
            bio: obj
                .get("bio")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            birthday: obj
                .get("birthday")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            birthday_public: obj
                .get("birthdayPublic")
                .or_else(|| obj.get("birthday_public"))
                .and_then(|v| v.as_bool()),
            quirk: obj
                .get("quirk")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            is_bot: obj
                .get("isBot")
                .or_else(|| obj.get("is_bot"))
                .and_then(|v| v.as_bool())
                .or(Some(false)),
            gender: obj
                .get("gender")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            metadata: obj.get("metadata").cloned(),
            raw_data: Some(data.clone()),
            last_seen,
            message_count: 0,
            deleted_count: 0,
            recalled_count: 0,
            created_at: now,
            updated_at: now,
        })
    }
}
