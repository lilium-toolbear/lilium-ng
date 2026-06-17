use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 models/ingestion/dzmm_account.py
pub type DzmmAccount = Model;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "dzmm_account")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: String,
    pub user_profile: serde_json::Value,
    pub email: Option<String>,
    pub password: Option<String>,
    pub signin_code: Option<String>,
    pub signin_code_image: Option<Vec<u8>>,
    pub signin_code_image_mime: Option<String>,
    pub cookies: Option<String>,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
