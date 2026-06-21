use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Python parity source: dzmm_archive@fea92bfdbe3ae0e0ce117fd0b8785099f77b0050 models/ingestion/websocket_event.py
pub type WebSocketEvent = websocket_event::Model;
// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 models/ingestion/event_processor_offset.py
pub type EventProcessorOffset = event_processor_offset::Model;

pub mod websocket_event {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "websocket_events")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: i64,
        #[sea_orm(primary_key, column_name = "timestamp", auto_increment = false)]
        pub timestamp: DateTime<Utc>,
        pub user_id: Uuid,
        pub event: String,
        pub data: serde_json::Value,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub account_user_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub received_at: DateTime<Utc>,
    pub source: String,
}

pub mod event_processor_offset {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "event_processor_offsets")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub processor_id: String,
        pub last_processed_id: i64,
        pub last_processed_timestamp: Option<DateTime<Utc>>,
        pub last_processed_at: Option<DateTime<Utc>>,
        pub updated_at: DateTime<Utc>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
