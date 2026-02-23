mod auth;
mod commands;
mod db;
mod entity;
mod error;
mod handlers;
mod matrix;
mod state;

use axum::Router;
use axum::routing::{get, post, delete};
use tower_http::cors::CorsLayer;

use crate::matrix::MatrixClient;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:uwu-admin.db?mode=rwc".into());

    let db = db::init_db(&database_url)
        .await
        .expect("Failed to initialize database");

    // Generate random JWT secret
    let jwt_secret: Vec<u8> = {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..32).map(|_| rng.r#gen::<u8>()).collect()
    };

    let state = AppState::new(db, jwt_secret);

    // Restore all saved servers
    match db::load_all_servers(&state.db).await {
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

    let app = Router::new()
        // Auth routes (unauthenticated)
        .route("/api/auth/status", get(auth::auth_status))
        .route("/api/auth/setup", post(auth::setup))
        .route("/api/auth/login", post(auth::login))
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
            "/api/servers/{server_id}/rooms/{room_id}",
            get(handlers::room_info),
        )
        .route(
            "/api/servers/{server_id}/server/status",
            get(handlers::server_status),
        )
        .route(
            "/api/servers/{server_id}/server/uptime",
            get(handlers::server_uptime),
        )
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Failed to bind to port 3001");

    tracing::info!("uwu-admin-api listening on :3001");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
