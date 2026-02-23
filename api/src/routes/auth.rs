use std::time::Duration;

use axum::Router;
use axum::error_handling::HandleErrorLayer;
use axum::http::StatusCode;
use axum::routing::{get, post};
use tower::ServiceBuilder;

use crate::handlers;
use crate::state::SharedState;

pub fn auth_routes() -> Router<SharedState> {
    Router::new()
        .route("/api/auth/status", get(handlers::auth::auth_status))
        .route("/api/auth/setup", post(handlers::auth::setup))
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/auth/refresh", post(handlers::auth::refresh))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(
                    |_: Box<dyn std::error::Error + Send + Sync>| async {
                        StatusCode::TOO_MANY_REQUESTS
                    },
                ))
                .buffer(100)
                .rate_limit(5, Duration::from_secs(60)),
        )
}
