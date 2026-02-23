use axum::Router;
use axum::routing::{get, post};

use crate::handlers;
use crate::state::SharedState;

pub fn auth_routes() -> Router<SharedState> {
    let rate_limited = Router::new()
        .route("/api/auth/setup", post(handlers::auth::setup))
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/auth/refresh", post(handlers::auth::refresh));

    #[cfg(not(test))]
    let rate_limited = {
        use std::sync::Arc;
        use tower_governor::GovernorLayer;
        use tower_governor::governor::GovernorConfigBuilder;

        let governor_conf = Arc::new(
            GovernorConfigBuilder::default()
                .per_second(12)
                .burst_size(5)
                .finish()
                .unwrap(),
        );

        rate_limited.layer(GovernorLayer::new(governor_conf))
    };

    Router::new()
        .route("/api/auth/status", get(handlers::auth::auth_status))
        .merge(rate_limited)
}
