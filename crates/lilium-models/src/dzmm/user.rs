use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Python parity source: dzmm_archive@6a92a9914602d633ff6fa3f5908fa68d00c36fcd models/dzmm/user.py
// Parity decision: Python exposes `name_tsv`, but Rust intentionally keeps
// PostgreSQL search vectors out of the table row model. The DB column still
// exists and is only referenced from search predicates.
pub type User = Model;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
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

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

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
