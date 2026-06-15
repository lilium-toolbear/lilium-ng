use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "outgoing_commands")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub created_at: DateTimeUtc,
    pub account_user_id: String,
    pub event: String,
    pub data: Json,
    pub require_ack: bool,
    pub status: String,
    pub processed_at: Option<DateTimeUtc>,
    pub ack_response: Option<Json>,
    pub error_message: Option<String>,
    pub attempt_count: i32,
    pub max_attempts: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
