use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "event_processor_offsets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub processor_id: String,
    pub last_processed_id: i32,
    pub last_processed_at: Option<DateTimeUtc>,
    pub updated_at: DateTimeUtc,
    pub last_processed_timestamp: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
