use sea_orm::{
    ActiveModelTrait, ConnectionTrait, Database, DatabaseConnection, EntityTrait, IntoActiveModel,
    Set, Statement,
};

use crate::entity::{admin_user, server};
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

pub fn load_secret_from_env(env_var: &str) -> Vec<u8> {
    match std::env::var(env_var) {
        Ok(hex) if !hex.is_empty() => {
            let bytes = hex_to_bytes(&hex)
                .unwrap_or_else(|e| panic!("{env_var} is not valid hex: {e}"));
            if bytes.len() != 32 {
                panic!("{env_var} must be 32 bytes (64 hex chars), got {}", bytes.len());
            }
            tracing::info!("{env_var} loaded from environment");
            bytes
        }
        _ => {
            use rand::Rng;
            let secret: Vec<u8> = {
                let mut rng = rand::thread_rng();
                (0..32).map(|_| rng.r#gen::<u8>()).collect()
            };
            tracing::warn!(
                "{env_var} not set — generated random secret (sessions will not survive restarts)",
            );
            secret
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
        if crate::crypto::is_encrypted(&s.access_token) {
            s.access_token = crate::crypto::decrypt(encryption_key, &s.access_token)
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
        if crate::crypto::is_encrypted(&s.access_token) {
            s.access_token = crate::crypto::decrypt(encryption_key, &s.access_token)
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
    let encrypted_token = crate::crypto::encrypt(encryption_key, access_token)
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

    if let Some(server) = server {
        let mut active: server::ActiveModel = server.into_active_model();
        active.access_token = Set(encrypted_token.to_owned());
        active
            .update(db)
            .await
            .map_err(|e| ApiError::DbError(e.to_string()))?;
    }

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

    if let Some(server) = server {
        let mut active: server::ActiveModel = server.into_active_model();
        active.since = Set(Some(since.to_owned()));
        active
            .update(db)
            .await
            .map_err(|e| ApiError::DbError(e.to_string()))?;
    }

    Ok(())
}
