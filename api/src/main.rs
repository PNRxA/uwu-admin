mod auth;
mod commands;
mod crypto;
mod db;
mod entity;
mod error;
mod handlers;
mod matrix;
mod state;
mod validation;

use std::time::Duration;

use axum::Router;
use axum::error_handling::HandleErrorLayer;
use axum::http::{HeaderName, Method, StatusCode};
use axum::routing::{get, post, delete};
use tower::ServiceBuilder;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::matrix::MatrixClient;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Load .env file if present (useful for development)
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:uwu-admin.db?mode=rwc".into());

    let db = db::init_db(&database_url)
        .await
        .expect("Failed to initialize database");

    let jwt_secret = db::load_secret_from_env("JWT_SECRET");
    let encryption_key = db::load_secret_from_env("ENCRYPTION_KEY");

    // Seed admin user from environment if set and no admin exists yet
    if let (Ok(username), Ok(password)) = (
        std::env::var("ADMIN_USERNAME"),
        std::env::var("ADMIN_PASSWORD"),
    ) {
        if !username.is_empty() && !password.is_empty() {
            match db::count_admin_users(&db).await {
                Ok(0) => {
                    if let Err(e) = validation::validate_username(&username) {
                        panic!("ADMIN_USERNAME is invalid: {e}");
                    }
                    if let Err(e) = validation::validate_password(&password) {
                        panic!("ADMIN_PASSWORD is invalid: {e}");
                    }
                    let hash = auth::hash_password(&password)
                        .expect("Failed to hash ADMIN_PASSWORD");
                    db::create_admin_user(&db, &username, &hash)
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

    // Migrate any plaintext access tokens to encrypted
    match db::load_all_servers_raw(&db).await {
        Ok(servers) => {
            for server in &servers {
                if !crypto::is_encrypted(&server.access_token) {
                    match crypto::encrypt(&encryption_key, &server.access_token) {
                        Ok(encrypted) => {
                            if let Err(e) = db::update_server_token(&db, server.id, &encrypted).await {
                                tracing::warn!("Failed to migrate token for server {}: {e}", server.id);
                            } else {
                                tracing::info!("Migrated plaintext token to encrypted for server {}", server.id);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to encrypt token for server {}: {e}", server.id);
                        }
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to load servers for token migration: {e}");
        }
    }

    // Clean up expired refresh tokens
    match db::delete_expired_refresh_tokens(&db).await {
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
    match db::load_all_servers(&state.db, &state.encryption_key).await {
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
                        state.clients.lock().await.insert(server.id, client);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Server {} session invalid, removing: {e}",
                            server.id
                        );
                        if let Err(e) = db::delete_server(&state.db, server.id).await {
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

    let auth_routes = Router::new()
        .route("/api/auth/status", get(auth::auth_status))
        .route("/api/auth/setup", post(auth::setup))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/refresh", post(auth::refresh))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|_: Box<dyn std::error::Error + Send + Sync>| async {
                    StatusCode::TOO_MANY_REQUESTS
                }))
                .buffer(100)
                .rate_limit(5, Duration::from_secs(60)),
        );

    let app = Router::new()
        .merge(auth_routes)
        .route("/api/auth/logout", post(auth::logout))
        // Server management
        .route("/api/servers", post(handlers::add_server).get(handlers::list_servers))
        .route("/api/servers/{server_id}", delete(handlers::remove_server))
        // Server-scoped routes
        .route("/api/servers/{server_id}/command", post(handlers::command))
        .route(
            "/api/servers/{server_id}/users",
            get(handlers::list_users).post(handlers::create_user),
        )
        .route("/api/servers/{server_id}/rooms", get(handlers::list_rooms))
        .route(
            "/api/servers/{server_id}/server/status",
            get(handlers::server_status),
        )
        .route(
            "/api/servers/{server_id}/server/uptime",
            get(handlers::server_uptime),
        )
        .layer({
            let cors = CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::DELETE])
                .allow_headers([
                    HeaderName::from_static("content-type"),
                    HeaderName::from_static("authorization"),
                ]);
            match std::env::var("CORS_ORIGIN") {
                Ok(origin) => cors.allow_origin(origin.parse::<axum::http::HeaderValue>().expect("Invalid CORS_ORIGIN")),
                Err(_) => cors.allow_origin(AllowOrigin::default()),
            }
        })
        .with_state(state);

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
