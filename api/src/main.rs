mod error;
mod handlers;
mod matrix;
mod state;

use axum::Router;
use axum::routing::{get, post};
use tower_http::cors::CorsLayer;

use crate::state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState::new();

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
