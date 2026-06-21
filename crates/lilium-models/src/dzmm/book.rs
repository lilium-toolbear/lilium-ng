use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Python parity source: dzmm_archive@227bc1179 models/dzmm/book.py
pub type Book = Model;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "books")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub book_id: Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub slug: Option<String>,
    pub is_nsfw: bool,
    pub is_public: bool,
    pub cover_image_url: Option<String>,
    pub local_cover_path: Option<String>,
    pub user_id: Option<Uuid>,
    pub author: Option<serde_json::Value>,
    pub chapter_count: i32,
    pub total_word_count: i32,
    pub latest_chapter: Option<serde_json::Value>,
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
    /// Create a Book from an explore-feed API dict. Mirrors Python
    /// `Book.from_api`. Requires `id`. Accepts snake_case and camelCase.
    pub fn from_api(data: &serde_json::Value) -> Option<Self> {
        let book_id = data
            .get("id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())?;
        let now = Utc::now();
        Some(Self {
            book_id,
            title: data
                .get("title")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            description: data
                .get("description")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            slug: data.get("slug").and_then(|v| v.as_str()).map(str::to_owned),
            is_nsfw: super::bool_field(data, "is_nsfw", "isNsfw", false),
            is_public: super::bool_field(data, "is_public", "isPublic", true),
            cover_image_url: data
                .get("cover_image_url")
                .and_then(|v| v.as_str())
                .or_else(|| data.get("coverImageUrl").and_then(|v| v.as_str()))
                .map(str::to_owned),
            local_cover_path: None,
            user_id: data
                .get("user_id")
                .and_then(|v| v.as_str())
                .or_else(|| data.get("userId").and_then(|v| v.as_str()))
                .or_else(|| data.get("authorId").and_then(|v| v.as_str()))
                .and_then(|s| Uuid::parse_str(s).ok()),
            author: normalize_author(data),
            chapter_count: super::int_field(data, "chapter_count", "chapterCount", 0),
            total_word_count: super::int_field(data, "total_word_count", "totalWordCount", 0),
            latest_chapter: data
                .get("latest_chapter")
                .or_else(|| data.get("latestChapter"))
                .cloned(),
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

/// Normalize the `author` field like Python: string → `{full_name}`, or
/// build from `authorName`/`authorAvatar` when absent.
pub(crate) fn normalize_author(data: &serde_json::Value) -> Option<serde_json::Value> {
    if let Some(s) = data.get("author").and_then(|v| v.as_str()) {
        return Some(serde_json::json!({"full_name": s}));
    }
    if let Some(obj) = data.get("author").filter(|v| v.is_object()) {
        return Some(obj.clone());
    }
    if let Some(name) = data.get("authorName").and_then(|v| v.as_str()) {
        let avatar = data
            .get("authorAvatar")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        return Some(serde_json::json!({"full_name": name, "avatar_url": avatar}));
    }
    None
}
