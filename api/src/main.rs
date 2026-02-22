mod db;
mod error;
mod handlers;
mod matrix;
mod state;

use axum::Router;
use axum::routing::{get, post};
use tower_http::cors::CorsLayer;

use crate::matrix::MatrixClient;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:uwu-admin.db?mode=rwc".into());

    let pool = db::init_pool(&database_url)
        .await
        .expect("Failed to initialize database");

    let state = AppState::new(pool);

    // Attempt to restore a saved session
    match db::load_session(&state.db).await {
        Ok(Some(session)) => {
            match MatrixClient::restore(
                session.homeserver,
                session.access_token,
                session.room_id,
                session.user_id.clone(),
                session.since,
            )
            .await
            {
                Ok(client) => {
                    tracing::info!("Session restored for {}", session.user_id);
                    *state.client.lock().await = Some(client);
                }
                Err(e) => {
                    tracing::warn!("Saved session invalid, deleting: {e}");
                    if let Err(e) = db::delete_session(&state.db).await {
                        tracing::warn!("Failed to delete stale session: {e}");
                    }
                }
            }
        }
        Ok(None) => {
            tracing::info!("No saved session found");
        }
        Err(e) => {
            tracing::warn!("Failed to load session from database: {e}");
        }
    }

    let app = Router::new()
        .route("/api/connect", post(handlers::connect))
        .route("/api/disconnect", post(handlers::disconnect))
        .route("/api/status", get(handlers::status))
        .route("/api/command", post(handlers::command))
        .route("/api/users", get(handlers::list_users).post(handlers::create_user))
        .route("/api/rooms", get(handlers::list_rooms))
        .route("/api/rooms/{room_id}", get(handlers::room_info))
        .route("/api/server/status", get(handlers::server_status))
        .route("/api/server/uptime", get(handlers::server_uptime))
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
