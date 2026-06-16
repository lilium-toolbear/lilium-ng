use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Python parity source: dzmm_archive@6a92a9914602d633ff6fa3f5908fa68d00c36fcd models/dzmm/user_history.py
pub type UserHistory = Model;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub birthday: Option<String>,
    pub birthday_public: Option<bool>,
    pub quirk: Option<String>,
    pub is_bot: Option<bool>,
    pub gender: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub raw_data: Option<serde_json::Value>,
    pub recorded_at: DateTime<Utc>,
    pub avatar_file: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
