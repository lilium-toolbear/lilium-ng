use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "dzmm_account")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: String,
    pub user_profile: Json,
    pub email: Option<String>,
    pub password: Option<String>,
    pub signin_code: Option<String>,
    pub cookies: Option<String>,
    pub is_enabled: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub signin_code_image: Option<Vec<u8>>,
    pub signin_code_image_mime: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
