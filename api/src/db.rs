use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;

use crate::error::ApiError;

#[derive(sqlx::FromRow)]
pub struct SessionRow {
    pub homeserver: String,
    pub access_token: String,
    pub room_id: String,
    pub user_id: String,
    pub since: Option<String>,
}

pub async fn init_pool(database_url: &str) -> Result<SqlitePool, ApiError> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sessions (
            id           INTEGER PRIMARY KEY CHECK (id = 1),
            homeserver   TEXT NOT NULL,
            access_token TEXT NOT NULL,
            room_id      TEXT NOT NULL,
            user_id      TEXT NOT NULL,
            since        TEXT
        )",
    )
    .execute(&pool)
    .await
    .map_err(|e| ApiError::DbError(e.to_string()))?;

    Ok(pool)
}

pub async fn load_session(pool: &SqlitePool) -> Result<Option<SessionRow>, ApiError> {
    let row = sqlx::query_as::<_, SessionRow>("SELECT homeserver, access_token, room_id, user_id, since FROM sessions WHERE id = 1")
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    Ok(row)
}

pub async fn save_session(
    pool: &SqlitePool,
    homeserver: &str,
    access_token: &str,
    room_id: &str,
    user_id: &str,
    since: Option<&str>,
) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO sessions (id, homeserver, access_token, room_id, user_id, since)
         VALUES (1, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
            homeserver = excluded.homeserver,
            access_token = excluded.access_token,
            room_id = excluded.room_id,
            user_id = excluded.user_id,
            since = excluded.since",
    )
    .bind(homeserver)
    .bind(access_token)
    .bind(room_id)
    .bind(user_id)
    .bind(since)
    .execute(pool)
    .await
    .map_err(|e| ApiError::DbError(e.to_string()))?;

    Ok(())
}

pub async fn delete_session(pool: &SqlitePool) -> Result<(), ApiError> {
    sqlx::query("DELETE FROM sessions WHERE id = 1")
        .execute(pool)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    Ok(())
}

pub async fn update_since(pool: &SqlitePool, since: &str) -> Result<(), ApiError> {
    sqlx::query("UPDATE sessions SET since = ? WHERE id = 1")
        .bind(since)
        .execute(pool)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    Ok(())
}
