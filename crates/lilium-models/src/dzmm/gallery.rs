use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Python parity source: dzmm_archive@227bc1179 models/dzmm/gallery.py
pub type Gallery = Model;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "galleries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub gallery_id: Uuid,
    pub title: Option<String>,
    pub user_id: Option<Uuid>,
    pub user_full_name: Option<String>,
    pub user_avatar_url: Option<String>,
    pub images: Option<Vec<String>>,
    pub local_image_paths: Option<Vec<String>>,
    pub likes_count: i32,
    pub dislikes_count: i32,
    pub comments_count: i32,
    pub top_comments: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub fetched_at: DateTime<Utc>,
    pub raw_data: Option<serde_json::Value>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Create a Gallery from an explore-feed API dict. Mirrors Python
    /// `Gallery.from_api`. Requires `id` and `authorId`.
    pub fn from_api(data: &serde_json::Value) -> Option<Self> {
        let gallery_id = data
            .get("id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())?;
        let user_id = data
            .get("authorId")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())?;
        let author = data.get("author")?;
        let now = Utc::now();
        Some(Self {
            gallery_id,
            title: data
                .get("title")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            user_id: Some(user_id),
            user_full_name: author
                .get("fullName")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            user_avatar_url: author
                .get("avatarUrl")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            images: data.get("images").and_then(|v| v.as_array()).map(|arr| {
                arr.iter()
                    .filter_map(|t| t.as_str().map(str::to_owned))
                    .collect()
            }),
            local_image_paths: None,
            likes_count: super::int_field(data, "likes_count", "likesCount", 0),
            dislikes_count: super::int_field(data, "dislikes_count", "dislikesCount", 0),
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
            fetched_at: now,
            raw_data: Some(data.clone()),
        })
    }
}
