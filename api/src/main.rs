mod entity;
mod error;
mod handlers;
mod routes;
mod services;
mod state;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::services::matrix::MatrixClient;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Load .env file if present (useful for development)
    dotenvy::dotenv().ok();

    services::commands::init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:uwu-admin.db?mode=rwc".into());

    let db = services::db::init_db(&database_url)
        .await
        .expect("Failed to initialize database");

    let jwt_secret = services::db::load_secret_from_env("JWT_SECRET");
    let encryption_key = services::db::load_secret_from_env("ENCRYPTION_KEY");

    // Seed admin user from environment if set and no admin exists yet
    if let (Ok(username), Ok(password)) = (
        std::env::var("ADMIN_USERNAME"),
        std::env::var("ADMIN_PASSWORD"),
    ) {
        if !username.is_empty() && !password.is_empty() {
            match services::db::count_admin_users(&db).await {
                Ok(0) => {
                    if let Err(e) = services::validation::validate_username(&username) {
                        panic!("ADMIN_USERNAME is invalid: {e}");
                    }
                    if let Err(e) = services::validation::validate_password(&password) {
                        panic!("ADMIN_PASSWORD is invalid: {e}");
                    }
                    let hash = handlers::auth::hash_password(&password)
                        .expect("Failed to hash ADMIN_PASSWORD");
                    services::db::create_admin_user(&db, &username, &hash)
                        .await
                        .expect("Failed to create admin user from environment");
                    tracing::info!("Admin user '{}' created from environment variables", username);
                }
                Ok(_) => {
                    tracing::info!("Admin user already exists, skipping ADMIN_USERNAME/ADMIN_PASSWORD");
                }
                Err(e) => {
                    tracing::warn!("Failed to check admin users: {e}");
                }
            }
        }
    }

    // Migrate access tokens: plaintext → encrypted, legacy hex → prefixed
    match services::db::load_all_servers_raw(&db).await {
        Ok(servers) => {
            for server in &servers {
                if services::crypto::is_encrypted(&server.access_token) {
                    // Already has enc: prefix, nothing to do
                    continue;
                }

                if services::crypto::is_legacy_encrypted(&server.access_token) {
                    // Legacy format (all-hex, no prefix): decrypt then re-encrypt with prefix
                    match services::crypto::decrypt(&encryption_key, &server.access_token) {
                        Ok(plaintext) => match services::crypto::encrypt(&encryption_key, &plaintext) {
                            Ok(encrypted) => {
                                if let Err(e) = services::db::update_server_token(&db, server.id, &encrypted).await {
                                    tracing::warn!("Failed to re-encrypt token for server {}: {e}", server.id);
                                } else {
                                    tracing::info!("Migrated legacy encrypted token to prefixed for server {}", server.id);
                                }
                            }
                            Err(e) => tracing::warn!("Failed to re-encrypt token for server {}: {e}", server.id),
                        },
                        Err(e) => tracing::warn!("Failed to decrypt legacy token for server {}: {e}", server.id),
                    }
                } else {
                    // Plaintext token: encrypt with prefix
                    match services::crypto::encrypt(&encryption_key, &server.access_token) {
                        Ok(encrypted) => {
                            if let Err(e) = services::db::update_server_token(&db, server.id, &encrypted).await {
                                tracing::warn!("Failed to migrate token for server {}: {e}", server.id);
                            } else {
                                tracing::info!("Migrated plaintext token to encrypted for server {}", server.id);
                            }
                        }
                        Err(e) => tracing::warn!("Failed to encrypt token for server {}: {e}", server.id),
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to load servers for token migration: {e}");
        }
    }

    // Clean up expired refresh tokens
    match services::db::delete_expired_refresh_tokens(&db).await {
        Ok(count) if count > 0 => {
            tracing::info!("Cleaned up {count} expired refresh tokens");
        }
        Err(e) => {
            tracing::warn!("Failed to clean expired refresh tokens: {e}");
        }
        _ => {}
    }

    let state = AppState::new(db, jwt_secret, encryption_key);

    // Restore all saved servers
    match services::db::load_all_servers(&state.db, &state.encryption_key).await {
        Ok(servers) => {
            for server in servers {
                match MatrixClient::restore(
                    server.homeserver,
                    server.access_token,
                    server.room_id,
                    server.user_id.clone(),
                    server.since,
                )
                .await
                {
                    Ok(client) => {
                        tracing::info!("Server {} restored for {}", server.id, server.user_id);
                        state.clients.lock().await.insert(server.id, Arc::new(Mutex::new(client)));
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Server {} session invalid, removing: {e}",
                            server.id
                        );
                        if let Err(e) = services::db::delete_server(&state.db, server.id).await {
                            tracing::warn!("Failed to delete stale server {}: {e}", server.id);
                        }
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to load servers from database: {e}");
        }
    }

    let app = routes::build_router(state);

    let listen_addr =
        std::env::var("API_LISTEN").unwrap_or_else(|_| "127.0.0.1:3001".into());

    let listener = tokio::net::TcpListener::bind(&listen_addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {listen_addr}"));

    tracing::info!("uwu-admin-api listening on {listen_addr}");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

#[cfg(test)]
mod integration_tests;
