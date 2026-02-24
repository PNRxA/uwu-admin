use axum::Router;
use axum::routing::{delete, post};

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
}
