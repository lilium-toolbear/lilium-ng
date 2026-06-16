use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Python parity source: dzmm_archive@6a92a9914602d633ff6fa3f5908fa68d00c36fcd models/ingestion/outgoing_command.py
pub type OutgoingCommand = Model;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "outgoing_commands")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub account_user_id: String,
    pub event: String,
    pub data: serde_json::Value,
    pub require_ack: bool,
    pub status: String,
    pub processed_at: Option<DateTime<Utc>>,
    pub ack_response: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub attempt_count: i32,
    pub max_attempts: i32,
}

pub mod status {
    pub const PENDING: &str = "pending";
    pub const PROCESSING: &str = "processing";
    pub const SUCCESS: &str = "success";
    pub const FAILED: &str = "failed";
    pub const TIMEOUT: &str = "timeout";

    pub const TERMINAL_STATUSES: &[&str] = &[SUCCESS, FAILED, TIMEOUT];
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
