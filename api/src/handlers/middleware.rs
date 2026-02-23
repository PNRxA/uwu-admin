use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    middleware::Next,
    response::Response,
};

/// Defense-in-depth: on mutating requests (POST/PUT/DELETE), verify that the
/// `Origin` header (if present) matches the configured `CORS_ORIGIN`.
/// If `CORS_ORIGIN` is unset (dev mode) or the header is absent, allow through.
pub async fn validate_origin(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let dominated = matches!(req.method(), &Method::POST | &Method::PUT | &Method::DELETE);

    if dominated {
        if let Ok(allowed) = std::env::var("CORS_ORIGIN") {
            if let Some(origin) = req.headers().get("origin") {
                let origin_str = origin.to_str().unwrap_or("");
                if origin_str != allowed {
                    return Err(StatusCode::FORBIDDEN);
                }
            }
        }
    }

    Ok(next.run(req).await)
}
