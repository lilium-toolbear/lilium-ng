use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DzmmAccount {
    pub user_id: String,
    pub user_profile: serde_json::Value,
    pub email: Option<String>,
    pub password: Option<String>,
    pub signin_code: Option<String>,
    pub signin_code_image: Option<Vec<u8>>,
    pub signin_code_image_mime: Option<String>,
    pub cookies: Option<String>,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
