use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Python parity source: dzmm_archive@18fdefbc0b6979178d7f1eb4ce0624ec4a60a2f2 models/dzmm/chapter.py
pub type Chapter = Model;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "chapters")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub chapter_id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub is_adult: bool,
    pub is_nsfw: bool,
    pub user_id: Option<String>,
    pub author: Option<serde_json::Value>,
    pub likes_count: i32,
    pub comments_count: i32,
    pub top_comments: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub fetched_at: DateTime<Utc>,
    pub raw_data: Option<serde_json::Value>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Create a Chapter from an explore-feed API dict. Mirrors Python
    /// `Chapter.from_api`. Requires `id`. Accepts snake_case and camelCase.
    pub fn from_api(data: &serde_json::Value) -> Option<Self> {
        let chapter_id = data.get("id")?.as_str()?.to_string();
        let now = Utc::now();
        Some(Self {
            chapter_id,
            title: data
                .get("title")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            content: data
                .get("content")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            is_adult: super::bool_field(data, "is_adult", "isAdult", false),
            is_nsfw: super::bool_field(data, "is_nsfw", "isNsfw", false),
            user_id: data
                .get("user_id")
                .and_then(|v| v.as_str())
                .or_else(|| data.get("userId").and_then(|v| v.as_str()))
                .or_else(|| data.get("authorId").and_then(|v| v.as_str()))
                .map(str::to_owned),
            author: super::book::normalize_author(data),
            likes_count: super::int_field(data, "likes_count", "likesCount", 0),
            comments_count: super::int_field(data, "comments_count", "commentsCount", 0),
            top_comments: data
                .get("top_comments")
                .or_else(|| data.get("topComments"))
                .cloned(),
            created_at: super::parse_datetime(
                data.get("created_at")
                    .and_then(|v| v.as_str())
                    .or_else(|| data.get("createdAt").and_then(|v| v.as_str()))
                    .unwrap_or(""),
            )
            .unwrap_or(now),
            updated_at: data
                .get("updated_at")
                .and_then(|v| v.as_str())
                .or_else(|| data.get("updatedAt").and_then(|v| v.as_str()))
                .and_then(super::parse_datetime),
            published_at: data
                .get("published_at")
                .and_then(|v| v.as_str())
                .or_else(|| data.get("publishedAt").and_then(|v| v.as_str()))
                .and_then(super::parse_datetime),
            fetched_at: now,
            raw_data: Some(data.clone()),
        })
    }
}
