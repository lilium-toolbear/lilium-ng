use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 models/ingestion/websocket_connection.py
pub type WebsocketConnection = Model;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "websocket_connections")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub lock_id: i64,
    pub account_user_id: Uuid,
    pub connected_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
