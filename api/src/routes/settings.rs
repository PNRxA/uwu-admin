use axum::Router;
use axum::routing::get;

use crate::handlers;
use crate::state::SharedState;

pub fn settings_routes() -> Router<SharedState> {
    Router::new()
        .route(
            "/api/settings",
            get(handlers::settings::get_settings).put(handlers::settings::update_settings),
        )
}
