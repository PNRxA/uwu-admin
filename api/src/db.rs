use sea_orm::{
    ActiveModelTrait, ConnectionTrait, Database, DatabaseConnection, EntityTrait, IntoActiveModel,
    Set, Statement,
    sea_query::OnConflict,
};

use crate::entity::{self, ActiveModel, Column, Entity};
use crate::error::ApiError;

pub async fn init_db(database_url: &str) -> Result<DatabaseConnection, ApiError> {
    let db = Database::connect(database_url)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE TABLE IF NOT EXISTS sessions (
            id           INTEGER PRIMARY KEY CHECK (id = 1),
            homeserver   TEXT NOT NULL,
            access_token TEXT NOT NULL,
            room_id      TEXT NOT NULL,
            user_id      TEXT NOT NULL,
            since        TEXT
        )",
    ))
    .await
    .map_err(|e| ApiError::DbError(e.to_string()))?;

    Ok(db)
}

pub async fn load_session(
    db: &DatabaseConnection,
) -> Result<Option<entity::Model>, ApiError> {
    Entity::find_by_id(1)
        .one(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))
}

pub async fn save_session(
    db: &DatabaseConnection,
    homeserver: &str,
    access_token: &str,
    room_id: &str,
    user_id: &str,
    since: Option<&str>,
) -> Result<(), ApiError> {
    let model = ActiveModel {
        id: Set(1),
        homeserver: Set(homeserver.to_owned()),
        access_token: Set(access_token.to_owned()),
        room_id: Set(room_id.to_owned()),
        user_id: Set(user_id.to_owned()),
        since: Set(since.map(|s| s.to_owned())),
    };

    Entity::insert(model)
        .on_conflict(
            OnConflict::column(Column::Id)
                .update_columns([
                    Column::Homeserver,
                    Column::AccessToken,
                    Column::RoomId,
                    Column::UserId,
                    Column::Since,
                ])
                .to_owned(),
        )
        .exec(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    Ok(())
}

pub async fn delete_session(db: &DatabaseConnection) -> Result<(), ApiError> {
    Entity::delete_by_id(1)
        .exec(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    Ok(())
}

pub async fn update_since(db: &DatabaseConnection, since: &str) -> Result<(), ApiError> {
    let session = Entity::find_by_id(1)
        .one(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    if let Some(session) = session {
        let mut active: ActiveModel = session.into_active_model();
        active.since = Set(Some(since.to_owned()));
        active
            .update(db)
            .await
            .map_err(|e| ApiError::DbError(e.to_string()))?;
    }

    Ok(())
}
