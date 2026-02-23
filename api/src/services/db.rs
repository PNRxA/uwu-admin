use sea_orm::{
    ActiveModelTrait, ConnectionTrait, Database, DatabaseConnection, EntityTrait, IntoActiveModel,
    Set, Statement, TransactionTrait,
};

use zeroize::Zeroizing;

use crate::entity::{admin_user, refresh_token, server};
use crate::error::ApiError;

pub async fn init_db(database_url: &str) -> Result<DatabaseConnection, ApiError> {
    let db = Database::connect(database_url)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    // Create new tables
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE TABLE IF NOT EXISTS admin_users (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            username      TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            created_at    TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    ))
    .await
    .map_err(|e| ApiError::DbError(e.to_string()))?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE TABLE IF NOT EXISTS servers (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            homeserver   TEXT NOT NULL,
            access_token TEXT NOT NULL,
            room_id      TEXT NOT NULL,
            user_id      TEXT NOT NULL,
            since        TEXT
        )",
    ))
    .await
    .map_err(|e| ApiError::DbError(e.to_string()))?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE TABLE IF NOT EXISTS refresh_tokens (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id     INTEGER NOT NULL REFERENCES admin_users(id) ON DELETE CASCADE,
            token_hash  TEXT NOT NULL UNIQUE,
            expires_at  TEXT NOT NULL,
            created_at  TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    ))
    .await
    .map_err(|e| ApiError::DbError(e.to_string()))?;

    // Migrate old sessions table if it exists
    let has_sessions: Vec<sea_orm::QueryResult> = db
        .query_all(Statement::from_string(
            db.get_database_backend(),
            "SELECT name FROM sqlite_master WHERE type='table' AND name='sessions'",
        ))
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    if !has_sessions.is_empty() {
        db.execute(Statement::from_string(
            db.get_database_backend(),
            "INSERT INTO servers (homeserver, access_token, room_id, user_id, since)
             SELECT homeserver, access_token, room_id, user_id, since FROM sessions",
        ))
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

        db.execute(Statement::from_string(
            db.get_database_backend(),
            "DROP TABLE sessions",
        ))
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

        tracing::info!("Migrated sessions table to servers");
    }

    Ok(db)
}

// --- Secret loading from env ---

pub fn load_secret_from_env(env_var: &str) -> Zeroizing<Vec<u8>> {
    match std::env::var(env_var) {
        Ok(hex) if !hex.is_empty() => {
            let bytes = hex_to_bytes(&hex)
                .unwrap_or_else(|e| panic!("{env_var} is not valid hex: {e}"));
            if bytes.len() != 32 {
                panic!("{env_var} must be 32 bytes (64 hex chars), got {}", bytes.len());
            }
            tracing::info!("{env_var} loaded from environment");
            Zeroizing::new(bytes)
        }
        _ => {
            panic!(
                "{env_var} is not set. This is required.\n\
                 Generate one with: openssl rand -hex 32\n\
                 Then add it to your .env file."
            );
        }
    }
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Odd-length hex string".into());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

// --- Admin user CRUD ---

pub async fn count_admin_users(db: &DatabaseConnection) -> Result<u64, ApiError> {
    use sea_orm::PaginatorTrait;
    admin_user::Entity::find()
        .count(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))
}

pub async fn create_admin_user(
    db: &DatabaseConnection,
    username: &str,
    password_hash: &str,
) -> Result<admin_user::Model, ApiError> {
    let model = admin_user::ActiveModel {
        id: Default::default(),
        username: Set(username.to_owned()),
        password_hash: Set(password_hash.to_owned()),
        created_at: Set(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
    };

    let result = admin_user::Entity::insert(model)
        .exec(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    admin_user::Entity::find_by_id(result.last_insert_id)
        .one(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?
        .ok_or_else(|| ApiError::DbError("Failed to retrieve created user".into()))
}

/// Atomically check that no admin users exist and create the first one.
/// Returns `Err(Forbidden)` if an admin already exists (race-safe via transaction).
pub async fn create_first_admin_user(
    db: &DatabaseConnection,
    username: &str,
    password_hash: &str,
) -> Result<admin_user::Model, ApiError> {
    use sea_orm::PaginatorTrait;

    let txn = db
        .begin()
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    let count = admin_user::Entity::find()
        .count(&txn)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    if count > 0 {
        txn.rollback()
            .await
            .map_err(|e| ApiError::DbError(e.to_string()))?;
        return Err(ApiError::Forbidden("Setup already completed".into()));
    }

    let model = admin_user::ActiveModel {
        id: Default::default(),
        username: Set(username.to_owned()),
        password_hash: Set(password_hash.to_owned()),
        created_at: Set(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
    };

    let result = admin_user::Entity::insert(model)
        .exec(&txn)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    let user = admin_user::Entity::find_by_id(result.last_insert_id)
        .one(&txn)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?
        .ok_or_else(|| ApiError::DbError("Failed to retrieve created user".into()))?;

    txn.commit()
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    Ok(user)
}

pub async fn find_admin_user_by_username(
    db: &DatabaseConnection,
    username: &str,
) -> Result<Option<admin_user::Model>, ApiError> {
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;
    admin_user::Entity::find()
        .filter(admin_user::Column::Username.eq(username))
        .one(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))
}

pub async fn find_admin_user_by_id(
    db: &DatabaseConnection,
    id: i32,
) -> Result<Option<admin_user::Model>, ApiError> {
    admin_user::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))
}

// --- Refresh token CRUD ---

pub async fn create_refresh_token(
    db: &DatabaseConnection,
    user_id: i32,
    token_hash: &str,
    expires_at: &str,
) -> Result<refresh_token::Model, ApiError> {
    let model = refresh_token::ActiveModel {
        id: Default::default(),
        user_id: Set(user_id),
        token_hash: Set(token_hash.to_owned()),
        expires_at: Set(expires_at.to_owned()),
        created_at: Set(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
    };

    let result = refresh_token::Entity::insert(model)
        .exec(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    refresh_token::Entity::find_by_id(result.last_insert_id)
        .one(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?
        .ok_or_else(|| ApiError::DbError("Failed to retrieve created refresh token".into()))
}

pub async fn find_refresh_token_by_hash(
    db: &DatabaseConnection,
    token_hash: &str,
) -> Result<Option<refresh_token::Model>, ApiError> {
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;
    refresh_token::Entity::find()
        .filter(refresh_token::Column::TokenHash.eq(token_hash))
        .one(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))
}

pub async fn delete_refresh_token(db: &DatabaseConnection, id: i32) -> Result<(), ApiError> {
    refresh_token::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    Ok(())
}

pub async fn delete_refresh_tokens_for_user(
    db: &DatabaseConnection,
    user_id: i32,
) -> Result<(), ApiError> {
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;
    refresh_token::Entity::delete_many()
        .filter(refresh_token::Column::UserId.eq(user_id))
        .exec(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    Ok(())
}

pub async fn delete_expired_refresh_tokens(db: &DatabaseConnection) -> Result<u64, ApiError> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;
    let result = refresh_token::Entity::delete_many()
        .filter(refresh_token::Column::ExpiresAt.lt(now))
        .exec(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    Ok(result.rows_affected)
}

// --- Server CRUD ---

pub async fn load_all_servers(
    db: &DatabaseConnection,
    encryption_key: &[u8],
) -> Result<Vec<server::Model>, ApiError> {
    let mut servers = server::Entity::find()
        .all(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    for s in &mut servers {
        if super::crypto::is_encrypted(&s.access_token) {
            s.access_token = super::crypto::decrypt(encryption_key, &s.access_token)
                .map_err(|e| ApiError::DbError(format!("Failed to decrypt access token: {e}")))?;
        }
    }

    Ok(servers)
}

pub async fn load_all_servers_raw(db: &DatabaseConnection) -> Result<Vec<server::Model>, ApiError> {
    server::Entity::find()
        .all(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))
}

#[allow(unused)]
pub async fn load_server(
    db: &DatabaseConnection,
    id: i32,
    encryption_key: &[u8],
) -> Result<Option<server::Model>, ApiError> {
    let mut server = server::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    if let Some(ref mut s) = server {
        if super::crypto::is_encrypted(&s.access_token) {
            s.access_token = super::crypto::decrypt(encryption_key, &s.access_token)
                .map_err(|e| ApiError::DbError(format!("Failed to decrypt access token: {e}")))?;
        }
    }

    Ok(server)
}

pub async fn save_server(
    db: &DatabaseConnection,
    homeserver: &str,
    access_token: &str,
    room_id: &str,
    user_id: &str,
    since: Option<&str>,
    encryption_key: &[u8],
) -> Result<i32, ApiError> {
    let encrypted_token = super::crypto::encrypt(encryption_key, access_token)
        .map_err(|e| ApiError::DbError(format!("Failed to encrypt access token: {e}")))?;

    let model = server::ActiveModel {
        id: Default::default(),
        homeserver: Set(homeserver.to_owned()),
        access_token: Set(encrypted_token),
        room_id: Set(room_id.to_owned()),
        user_id: Set(user_id.to_owned()),
        since: Set(since.map(|s| s.to_owned())),
    };

    let result = server::Entity::insert(model)
        .exec(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    Ok(result.last_insert_id)
}

pub async fn update_server_token(
    db: &DatabaseConnection,
    id: i32,
    encrypted_token: &str,
) -> Result<(), ApiError> {
    let server = server::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    let server = server.ok_or_else(|| ApiError::DbError(format!("Server {id} not found")))?;
    let mut active: server::ActiveModel = server.into_active_model();
    active.access_token = Set(encrypted_token.to_owned());
    active
        .update(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    Ok(())
}

pub async fn delete_server(db: &DatabaseConnection, id: i32) -> Result<(), ApiError> {
    server::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    Ok(())
}

pub async fn update_server_since(
    db: &DatabaseConnection,
    id: i32,
    since: &str,
) -> Result<(), ApiError> {
    let server = server::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    let server = server.ok_or_else(|| ApiError::DbError(format!("Server {id} not found")))?;
    let mut active: server::ActiveModel = server.into_active_model();
    active.since = Set(Some(since.to_owned()));
    active
        .update(db)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    async fn test_db() -> DatabaseConnection {
        init_db("sqlite::memory:").await.expect("in-memory DB")
    }

    fn test_encryption_key() -> Vec<u8> {
        vec![0xCC; 32]
    }

    #[tokio::test]
    #[serial]
    async fn create_and_find_admin_user() {
        let db = test_db().await;
        let user = create_admin_user(&db, "alice", "hash123").await.unwrap();
        assert_eq!(user.username, "alice");
        assert_eq!(user.password_hash, "hash123");

        let found = find_admin_user_by_username(&db, "alice").await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, user.id);
        assert_eq!(found.username, "alice");
    }

    #[tokio::test]
    #[serial]
    async fn find_admin_user_not_found() {
        let db = test_db().await;
        let found = find_admin_user_by_username(&db, "nonexistent").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn save_and_load_server() {
        let db = test_db().await;
        let key = test_encryption_key();
        let id = save_server(&db, "https://matrix.example.com", "tok_abc", "!room:host", "@bot:host", Some("s123"), &key)
            .await
            .unwrap();

        let servers = load_all_servers(&db, &key).await.unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].id, id);
        assert_eq!(servers[0].homeserver, "https://matrix.example.com");
        assert_eq!(servers[0].access_token, "tok_abc");
        assert_eq!(servers[0].room_id, "!room:host");
        assert_eq!(servers[0].user_id, "@bot:host");
        assert_eq!(servers[0].since.as_deref(), Some("s123"));
    }

    #[tokio::test]
    #[serial]
    async fn update_server_token_success() {
        let db = test_db().await;
        let key = test_encryption_key();
        let id = save_server(&db, "https://hs", "old_tok", "!r:h", "@u:h", None, &key)
            .await
            .unwrap();

        let new_encrypted = super::super::crypto::encrypt(&key, "new_tok").unwrap();
        update_server_token(&db, id, &new_encrypted).await.unwrap();

        let server = load_server(&db, id, &key).await.unwrap().unwrap();
        assert_eq!(server.access_token, "new_tok");
    }

    #[tokio::test]
    #[serial]
    async fn update_server_token_not_found() {
        let db = test_db().await;
        let result = update_server_token(&db, 9999, "enc_tok").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn update_server_since_success() {
        let db = test_db().await;
        let key = test_encryption_key();
        let id = save_server(&db, "https://hs", "tok", "!r:h", "@u:h", None, &key)
            .await
            .unwrap();

        update_server_since(&db, id, "s456").await.unwrap();

        let server = load_server(&db, id, &key).await.unwrap().unwrap();
        assert_eq!(server.since.as_deref(), Some("s456"));
    }

    #[tokio::test]
    #[serial]
    async fn update_server_since_not_found() {
        let db = test_db().await;
        let result = update_server_since(&db, 9999, "s789").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn delete_server_succeeds() {
        let db = test_db().await;
        let key = test_encryption_key();
        let id = save_server(&db, "https://hs", "tok", "!r:h", "@u:h", None, &key)
            .await
            .unwrap();

        delete_server(&db, id).await.unwrap();

        let servers = load_all_servers(&db, &key).await.unwrap();
        assert!(servers.is_empty());
    }
}
