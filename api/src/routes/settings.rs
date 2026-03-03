use axum::Router;
use axum::routing::get;

use crate::handlers;
use crate::state::SharedState;

pub fn public_settings_routes() -> Router<SharedState> {
    Router::new().route(
        "/api/settings/public",
        get(handlers::settings::get_public_settings),
    )
}

pub fn protected_settings_routes() -> Router<SharedState> {
    Router::new().route(
        "/api/settings",
        get(handlers::settings::get_settings).put(handlers::settings::update_settings),
    )
}
