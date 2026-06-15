use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub message_id: String,
    pub room_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sent_at: DateTimeUtc,
    pub sent_by: String,
    pub content_type: String,
    pub content_text: Option<String>,
    pub attachment_url: Option<String>,
    pub attachment_file: Option<String>,
    pub sticker_id: Option<String>,
    pub alt_text: Option<String>,
    pub metadata: Option<Json>,
    pub raw_data: Json,
    pub source: String,
    pub created_at: DateTimeUtc,
    pub updated_at: Option<DateTimeUtc>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTimeUtc>,
    pub deleted_by: Option<String>,
    pub is_recalled: bool,
    pub is_edited: bool,
    pub history: Option<Json>,
    pub reference_message_id: Option<String>,
    pub reference_data: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
