use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
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
    pub last_message_at: Option<DateTimeUtc>,
    pub first_message_at: Option<DateTimeUtc>,
    pub backfill_until: Option<DateTimeUtc>,
    pub history_complete: bool,
    pub message_count: i32,
    pub deleted_count: i32,
    pub recalled_count: i32,
    pub edited_count: i32,
    pub image_count: i32,
    pub is_active: bool,
    pub dissolved_at: Option<DateTimeUtc>,
    pub raw_data: Option<Json>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub account_ids: Vec<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
