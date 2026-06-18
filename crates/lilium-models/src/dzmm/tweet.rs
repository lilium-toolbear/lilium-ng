use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Python parity source: dzmm_archive@18fdefbc0b6979178d7f1eb4ce0624ec4a60a2f2 models/dzmm/tweet.py
pub type Tweet = Model;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tweets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub tweet_id: String,
    pub user_id: Option<String>,
    pub content: Option<String>,
    pub media_urls: Option<Vec<String>>,
    pub local_media_paths: Option<Vec<String>>,
    pub source: Option<String>,
    pub tweet_type: Option<String>,
    pub parent_tweet_id: Option<String>,
    pub reply_to_tweet_id: Option<String>,
    pub reply_to_username: Option<String>,
    pub is_edited: bool,
    pub edit_history: Option<serde_json::Value>,
    pub post_id: Option<String>,
    pub draw_id: Option<String>,
    pub likes_count: i32,
    pub comments_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub fetched_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub raw_data: Option<serde_json::Value>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Create a Tweet from an explore-feed API dict. Mirrors Python
    /// `Tweet.from_api`. Requires `id` and `created_at`/`createdAt`.
    pub fn from_api(data: &serde_json::Value) -> Option<Self> {
        let tweet_id = data.get("id")?.as_str()?.to_string();
        let created_at_str = data
            .get("created_at")
            .and_then(|v| v.as_str())
            .or_else(|| data.get("createdAt").and_then(|v| v.as_str()))?;
        let created_at = super::parse_datetime(created_at_str).unwrap_or_else(Utc::now);
        let updated_at = data
            .get("updated_at")
            .and_then(|v| v.as_str())
            .or_else(|| data.get("updatedAt").and_then(|v| v.as_str()))
            .and_then(super::parse_datetime);
        let now = Utc::now();

        Some(Self {
            tweet_id,
            user_id: extract_user_id(data),
            content: data
                .get("content")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            media_urls: normalize_media_urls(data),
            local_media_paths: None,
            source: data
                .get("source")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            tweet_type: data
                .get("tweet_type")
                .and_then(|v| v.as_str())
                .or_else(|| data.get("type").and_then(|v| v.as_str()))
                .map(str::to_owned),
            parent_tweet_id: data
                .get("parent_tweet_id")
                .and_then(|v| v.as_str())
                .or_else(|| data.get("parentTweetId").and_then(|v| v.as_str()))
                .map(str::to_owned),
            reply_to_tweet_id: data
                .get("reply_to_tweet_id")
                .and_then(|v| v.as_str())
                .or_else(|| data.get("replyToTweetId").and_then(|v| v.as_str()))
                .map(str::to_owned),
            reply_to_username: data
                .get("reply_to_username")
                .and_then(|v| v.as_str())
                .or_else(|| data.get("replyToUsername").and_then(|v| v.as_str()))
                .map(str::to_owned),
            is_edited: super::bool_field(data, "is_edited", "isEdited", false),
            edit_history: data
                .get("edit_history")
                .or_else(|| data.get("editHistory"))
                .cloned(),
            post_id: data
                .get("post_id")
                .and_then(|v| v.as_str())
                .or_else(|| data.get("postId").and_then(|v| v.as_str()))
                .or_else(|| data.get("chatroomId").and_then(|v| v.as_str()))
                .map(str::to_owned),
            draw_id: data
                .get("draw_id")
                .and_then(|v| v.as_str())
                .or_else(|| data.get("drawId").and_then(|v| v.as_str()))
                .map(str::to_owned),
            likes_count: super::int_field(data, "likes_count", "likesCount", 0),
            comments_count: super::int_field(data, "comments_count", "commentsCount", 0),
            created_at,
            updated_at,
            fetched_at: now,
            is_deleted: super::bool_field(data, "is_deleted", "isDeleted", false),
            raw_data: Some(data.clone()),
        })
    }

    pub fn media_urls_list(&self) -> Vec<String> {
        self.media_urls.clone().unwrap_or_default()
    }
}

fn extract_user_id(data: &serde_json::Value) -> Option<String> {
    if let Some(v) = data.get("user_id").and_then(|v| v.as_str()) {
        return Some(v.to_owned());
    }
    if let Some(v) = data.get("authorId").and_then(|v| v.as_str()) {
        return Some(v.to_owned());
    }
    if let Some(v) = data.get("author_id").and_then(|v| v.as_str()) {
        return Some(v.to_owned());
    }
    data.get("user")
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_str())
        .or_else(|| {
            data.get("author")
                .and_then(|v| v.get("id"))
                .and_then(|v| v.as_str())
        })
        .map(str::to_owned)
}

fn normalize_media_urls(data: &serde_json::Value) -> Option<Vec<String>> {
    for key in &["media_urls", "mediaUrls", "images", "displayMedia"] {
        if let Some(arr) = data.get(*key).and_then(|v| v.as_array())
            && !arr.is_empty()
        {
            let urls: Vec<String> = arr
                .iter()
                .filter_map(|item| {
                    if let Some(s) = item.as_str() {
                        return Some(s.to_owned());
                    }
                    item.get("videoUrl")
                        .or_else(|| item.get("video_url"))
                        .and_then(|v| v.as_str())
                        .map(str::to_owned)
                        .or_else(|| item.get("url").and_then(|v| v.as_str()).map(str::to_owned))
                })
                .collect();
            if !urls.is_empty() {
                return Some(urls);
            }
        }
    }
    None
}
