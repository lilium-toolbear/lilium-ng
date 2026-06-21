use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Python parity source: dzmm_archive@227bc1179 models/dzmm/card.py
pub type Card = Model;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "cards")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub card_id: i32,
    pub name: Option<String>,
    pub card_filename: Option<String>,
    pub original_filename: Option<String>,
    pub creator: Option<String>,
    pub creator_notes: Option<String>,
    pub user_id: Option<Uuid>,
    pub creator_full_name: Option<String>,
    pub creator_avatar_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_public: bool,
    pub is_sensitive: bool,
    pub is_image_blur: bool,
    pub is_gamefy: bool,
    pub image_info: Option<serde_json::Value>,
    pub weighted_rating: Option<String>,
    pub popularity_score: Option<String>,
    pub likes_count: i32,
    pub comments_count: i32,
    pub top_comments: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub fetched_at: DateTime<Utc>,
    pub raw_data: Option<serde_json::Value>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Create a Card from an explore-feed API dict. Mirrors Python
    /// `Card.from_api`. Requires `id` (integer).
    pub fn from_api(data: &serde_json::Value) -> Option<Self> {
        let card_id = data.get("id")?.as_i64()? as i32;
        let now = Utc::now();
        Some(Self {
            card_id,
            name: data.get("name").and_then(|v| v.as_str()).map(str::to_owned),
            card_filename: data
                .get("cardFilename")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            original_filename: data
                .get("originalFilename")
                .and_then(|v| v.as_str())
                .or_else(|| data.get("originalfilename").and_then(|v| v.as_str()))
                .map(str::to_owned),
            creator: data
                .get("creator")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            creator_notes: data
                .get("creatorNotes")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            user_id: data
                .get("userId")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok()),
            creator_full_name: data
                .get("creatorFullName")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            creator_avatar_url: data
                .get("creatorAvatarUrl")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            tags: data.get("tags").and_then(|v| v.as_array()).map(|arr| {
                arr.iter()
                    .filter_map(|t| t.as_str().map(str::to_owned))
                    .collect()
            }),
            is_public: data
                .get("isPublic")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            is_sensitive: data
                .get("isSensitive")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            is_image_blur: data
                .get("isImageBlur")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            is_gamefy: data
                .get("isGamefy")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            image_info: data.get("imageInfo").cloned(),
            weighted_rating: data
                .get("weightedRating")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            popularity_score: data
                .get("popularityScore")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            likes_count: data
                .get("likesCount")
                .and_then(|v| v.as_i64())
                .map(|n| n as i32)
                .unwrap_or(0),
            comments_count: data
                .get("commentsCount")
                .and_then(|v| v.as_i64())
                .map(|n| n as i32)
                .unwrap_or(0),
            top_comments: data.get("topComments").cloned(),
            created_at: super::parse_datetime(
                data.get("createdAt").and_then(|v| v.as_str()).unwrap_or(""),
            )
            .unwrap_or(now),
            published_at: data
                .get("publishedAt")
                .and_then(|v| v.as_str())
                .and_then(super::parse_datetime),
            fetched_at: now,
            raw_data: Some(data.clone()),
        })
    }
}
