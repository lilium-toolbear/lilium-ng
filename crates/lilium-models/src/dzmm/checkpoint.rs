use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Python parity source: dzmm_archive@227bc1179 models/dzmm/checkpoint.py
pub type Checkpoint = Model;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "checkpoints")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub checkpoint_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_public: bool,
    pub user_id: Option<Uuid>,
    pub user_name: Option<String>,
    pub user_avatar_url: Option<String>,
    pub creator: Option<serde_json::Value>,
    pub rating_avg: Option<String>,
    pub rating_count: i32,
    pub review_status: Option<String>,
    pub share_code: Option<String>,
    pub character_cards: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub fetched_at: DateTime<Utc>,
    pub raw_data: Option<serde_json::Value>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Create a Checkpoint from an explore-feed API dict. Mirrors Python
    /// `Checkpoint.from_api`. Requires `id`.
    pub fn from_api(data: &serde_json::Value) -> Option<Self> {
        let checkpoint_id = data
            .get("id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())?;
        let now = Utc::now();
        let creator = data.get("creator").filter(|v| v.is_object()).cloned();
        Some(Self {
            checkpoint_id,
            name: data.get("name").and_then(|v| v.as_str()).map(str::to_owned),
            description: data
                .get("description")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            is_public: data
                .get("isPublic")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            user_id: data
                .get("userId")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok()),
            user_name: data
                .get("userName")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            user_avatar_url: data
                .get("userAvatarUrl")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            creator,
            rating_avg: data
                .get("ratingAvg")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            rating_count: data
                .get("ratingCount")
                .and_then(|v| v.as_i64())
                .map(|n| n as i32)
                .unwrap_or(0),
            review_status: data
                .get("reviewStatus")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            share_code: data
                .get("shareCode")
                .and_then(|v| v.as_str())
                .or_else(|| data.get("sharecode").and_then(|v| v.as_str()))
                .map(str::to_owned),
            character_cards: data.get("characterCards").cloned(),
            created_at: super::parse_datetime(
                data.get("createdAt").and_then(|v| v.as_str()).unwrap_or(""),
            )
            .unwrap_or(now),
            updated_at: data
                .get("updatedAt")
                .and_then(|v| v.as_str())
                .and_then(super::parse_datetime),
            fetched_at: now,
            raw_data: Some(data.clone()),
        })
    }
}
