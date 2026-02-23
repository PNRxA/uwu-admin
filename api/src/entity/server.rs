use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "servers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub homeserver: String,
    pub access_token: String,
    pub room_id: String,
    pub user_id: String,
    pub since: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
