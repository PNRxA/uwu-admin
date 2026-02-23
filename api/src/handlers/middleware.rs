use axum::{
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

use super::auth::{AuthUser, Claims};
use crate::state::SharedState;

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

pub async fn require_auth(
    State(state): State<SharedState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&state.jwt_secret),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    req.extensions_mut().insert(AuthUser {
        username: token_data.claims.sub,
    });

    Ok(next.run(req).await)
}
