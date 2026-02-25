mod auth;
mod servers;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderName, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::state::SharedState;

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .merge(auth::auth_routes())
        .merge(
            servers::protected_routes()
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    crate::handlers::middleware::require_auth,
                )),
        )
        .layer(DefaultBodyLimit::max(65_536))
        .layer(axum::middleware::from_fn(crate::handlers::middleware::validate_origin))
        .layer({
            let cors = CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::DELETE])
                .allow_headers([
                    HeaderName::from_static("content-type"),
                    HeaderName::from_static("authorization"),
                ]);
            match std::env::var("CORS_ORIGIN") {
                Ok(origin) => cors
                    .allow_origin(
                        origin
                            .parse::<axum::http::HeaderValue>()
                            .expect("Invalid CORS_ORIGIN"),
                    )
                    .allow_credentials(true),
                Err(_) => cors.allow_origin(AllowOrigin::default()),
            }
        })
        .with_state(state)
}
