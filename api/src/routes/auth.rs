use axum::Router;
use axum::routing::{get, post};

use crate::handlers;
use crate::state::SharedState;

pub fn auth_routes() -> Router<SharedState> {
    let router = Router::new()
        .route("/api/auth/status", get(handlers::auth::auth_status))
        .route("/api/auth/setup", post(handlers::auth::setup))
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/auth/refresh", post(handlers::auth::refresh));

    #[cfg(not(test))]
    let router = {
        use std::time::Duration;
        use axum::error_handling::HandleErrorLayer;
        use axum::http::StatusCode;
        use tower::ServiceBuilder;

        router.layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(
                    |_: Box<dyn std::error::Error + Send + Sync>| async {
                        StatusCode::TOO_MANY_REQUESTS
                    },
                ))
                .buffer(100)
                .rate_limit(5, Duration::from_secs(60)),
        )
    };

    router
}
