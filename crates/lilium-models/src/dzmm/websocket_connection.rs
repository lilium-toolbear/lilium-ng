use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Python parity source: dzmm_archive@6a92a9914602d633ff6fa3f5908fa68d00c36fcd models/ingestion/websocket_connection.py
pub type WebsocketConnection = Model;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "websocket_connections")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub lock_id: i64,
    pub account_user_id: String,
    pub connected_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
