use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Python parity source: dzmm_archive@227bc1179 models/dzmm/image_gps.py
// Parity note: after the migration the single `message_id` PK became a composite
// `(source_type, source_id)` PK where `source_type` is 'message' or 'tweet' and
// `source_id` is the UUID of the owning message/tweet row.
pub type ImageGps = Model;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "image_gps")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub source_type: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub source_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    #[sea_orm(column_name = "timestamp")]
    pub timestamp: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
