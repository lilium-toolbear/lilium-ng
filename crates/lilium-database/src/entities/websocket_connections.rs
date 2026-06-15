use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "websocket_connections")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub lock_id: i64,
    pub account_user_id: String,
    pub connected_at: DateTimeUtc,
    pub last_heartbeat: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
