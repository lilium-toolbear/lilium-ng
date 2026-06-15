use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub birthday: Option<String>,
    pub birthday_public: Option<bool>,
    pub quirk: Option<String>,
    pub is_bot: Option<bool>,
    pub gender: Option<String>,
    pub metadata: Option<Json>,
    pub raw_data: Option<Json>,
    pub last_seen: Option<DateTimeUtc>,
    pub message_count: i32,
    pub deleted_count: i32,
    pub recalled_count: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub avatar_file: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for lilium_models::dzmm::user::User {
    fn from(model: Model) -> Self {
        Self {
            user_id: model.user_id,
            full_name: model.full_name,
            avatar_url: model.avatar_url,
            avatar_file: model.avatar_file,
            bio: model.bio,
            birthday: model.birthday,
            birthday_public: model.birthday_public,
            quirk: model.quirk,
            is_bot: model.is_bot,
            gender: model.gender,
            metadata: model.metadata,
            raw_data: model.raw_data,
            last_seen: model.last_seen,
            message_count: model.message_count,
            deleted_count: model.deleted_count,
            recalled_count: model.recalled_count,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
