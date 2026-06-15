use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "room_members")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub room_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: String,
    pub role: Option<String>,
    pub joined_at: Option<DateTimeUtc>,
    pub raw_data: Option<Json>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub left_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for lilium_models::dzmm::room_member::RoomMember {
    fn from(model: Model) -> Self {
        Self {
            room_id: model.room_id,
            user_id: model.user_id,
            role: model.role,
            joined_at: model.joined_at,
            left_at: model.left_at,
            raw_data: model.raw_data,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
