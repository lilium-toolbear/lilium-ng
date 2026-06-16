use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Python parity source: dzmm_archive@6a92a9914602d633ff6fa3f5908fa68d00c36fcd models/dzmm/room.py
pub type Room = Model;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "rooms")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub room_id: String,
    pub title: String,
    pub chat_type: Option<String>,
    pub avatar_url: Option<String>,
    pub member_count: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub is_public: Option<bool>,
    pub creator_id: Option<String>,
    pub account_ids: Vec<String>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub first_message_at: Option<DateTime<Utc>>,
    pub backfill_until: Option<DateTime<Utc>>,
    pub history_complete: bool,
    pub message_count: i32,
    pub deleted_count: i32,
    pub recalled_count: i32,
    pub edited_count: i32,
    pub image_count: i32,
    pub is_active: bool,
    pub dissolved_at: Option<DateTime<Utc>>,
    pub raw_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
