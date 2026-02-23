use axum::Router;
use axum::routing::{delete, post};

use crate::handlers;
use crate::state::SharedState;

pub fn protected_routes() -> Router<SharedState> {
    let router = Router::new()
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
        );

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
                .rate_limit(30, Duration::from_secs(60)),
        )
    };

    router
}
