use std::time::Duration;

use axum::Router;
use axum::error_handling::HandleErrorLayer;
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use tower::ServiceBuilder;

use crate::handlers;
use crate::state::SharedState;

pub fn protected_routes() -> Router<SharedState> {
    Router::new()
        .route(
            "/api/auth/logout",
            post(handlers::auth::logout),
        )
        .route(
            "/api/servers",
            post(handlers::servers::add_server).get(handlers::servers::list_servers),
        )
        .route(
            "/api/servers/{server_id}",
            delete(handlers::servers::remove_server),
        )
        .route(
            "/api/servers/{server_id}/command",
            post(handlers::servers::command),
        )
        .route(
            "/api/servers/{server_id}/users",
            get(handlers::servers::list_users).post(handlers::servers::create_user),
        )
        .route(
            "/api/servers/{server_id}/rooms",
            get(handlers::servers::list_rooms),
        )
        .route(
            "/api/servers/{server_id}/server/status",
            get(handlers::servers::server_status),
        )
        .route(
            "/api/servers/{server_id}/server/uptime",
            get(handlers::servers::server_uptime),
        )
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(
                    |_: Box<dyn std::error::Error + Send + Sync>| async {
                        StatusCode::TOO_MANY_REQUESTS
                    },
                ))
                .buffer(100)
                .rate_limit(30, Duration::from_secs(60)),
        )
}
