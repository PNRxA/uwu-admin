use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::services::db;
use crate::state::{AppState, SharedState};

// --- Helpers ---

async fn test_state() -> SharedState {
    let db = db::init_db("sqlite::memory:")
        .await
        .expect("in-memory DB");
    let jwt_secret = vec![0xAA; 32];
    let encryption_key = vec![0xBB; 32];
    AppState::new(db, jwt_secret, encryption_key)
}

fn test_app_with_state(state: SharedState) -> Router {
    crate::routes::build_router(state)
}

async fn test_app() -> Router {
    let state = test_state().await;
    test_app_with_state(state)
}

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn post_json(uri: &str, body: &Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(body).unwrap()))
        .unwrap()
}

fn get_json(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

fn post_json_auth(uri: &str, body: &Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(serde_json::to_vec(body).unwrap()))
        .unwrap()
}

fn get_json_auth(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

/// Run setup and return (access_token, refresh_token)
async fn do_setup(app: &Router) -> (String, String) {
    let body = json!({"username": "admin", "password": "supersecret123"});
    let resp = app
        .clone()
        .oneshot(post_json("/api/auth/setup", &body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    (
        json["token"].as_str().unwrap().to_string(),
        json["refresh_token"].as_str().unwrap().to_string(),
    )
}

// ========== Auth flow ==========

#[tokio::test]
async fn auth_status_setup_required() {
    let app = test_app().await;
    let resp = app.oneshot(get_json("/api/auth/status")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["setup_required"], true);
}

#[tokio::test]
async fn setup_creates_admin() {
    let app = test_app().await;
    let body = json!({"username": "admin", "password": "supersecret123"});
    let resp = app
        .oneshot(post_json("/api/auth/setup", &body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(json["token"].is_string());
    assert!(json["refresh_token"].is_string());
}

#[tokio::test]
async fn setup_rejects_second_attempt() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    let body = json!({"username": "admin", "password": "supersecret123"});

    let resp = app
        .clone()
        .oneshot(post_json("/api/auth/setup", &body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
        .oneshot(post_json("/api/auth/setup", &body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn setup_rejects_empty_username() {
    let app = test_app().await;
    let body = json!({"username": "", "password": "supersecret123"});
    let resp = app
        .oneshot(post_json("/api/auth/setup", &body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn setup_rejects_short_password() {
    let app = test_app().await;
    let body = json!({"username": "admin", "password": "short"});
    let resp = app
        .oneshot(post_json("/api/auth/setup", &body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn login_valid_credentials() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    do_setup(&app).await;

    let body = json!({"username": "admin", "password": "supersecret123"});
    let resp = app
        .oneshot(post_json("/api/auth/login", &body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(json["token"].is_string());
    assert!(json["refresh_token"].is_string());
}

#[tokio::test]
async fn login_wrong_password() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    do_setup(&app).await;

    let body = json!({"username": "admin", "password": "wrongpassword"});
    let resp = app
        .oneshot(post_json("/api/auth/login", &body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_unknown_user() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    do_setup(&app).await;

    let body = json!({"username": "nobody", "password": "supersecret123"});
    let resp = app
        .oneshot(post_json("/api/auth/login", &body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_status_after_setup() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    do_setup(&app).await;

    let resp = app
        .oneshot(get_json("/api/auth/status"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["setup_required"], false);
}

// ========== Refresh tokens ==========

#[tokio::test]
async fn refresh_valid_token() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (_, refresh) = do_setup(&app).await;

    let body = json!({"refresh_token": refresh});
    let resp = app
        .oneshot(post_json("/api/auth/refresh", &body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(json["token"].is_string());
    assert!(json["refresh_token"].is_string());
}

#[tokio::test]
async fn refresh_token_single_use() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (_, refresh) = do_setup(&app).await;

    let body = json!({"refresh_token": refresh});
    // First use should succeed
    let resp = app
        .clone()
        .oneshot(post_json("/api/auth/refresh", &body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Second use should fail (single-use rotation)
    let resp = app
        .oneshot(post_json("/api/auth/refresh", &body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn expired_refresh_token_rejected() {
    let state = test_state().await;
    let app = test_app_with_state(state.clone());

    // Create admin user directly
    let hash = crate::handlers::auth::hash_password("password123").unwrap();
    let user = db::create_admin_user(&state.db, "admin", &hash)
        .await
        .unwrap();

    // Insert an expired refresh token directly
    let raw_token = "expired_token_value";
    let token_hash = crate::handlers::auth::hash_refresh_token(raw_token);
    let expired_at = "2020-01-01 00:00:00";
    db::create_refresh_token(&state.db, user.id, &token_hash, expired_at)
        .await
        .unwrap();

    let body = json!({"refresh_token": raw_token});
    let resp = app
        .oneshot(post_json("/api/auth/refresh", &body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ========== JWT protection ==========

#[tokio::test]
async fn no_jwt_returns_401() {
    let app = test_app().await;
    let resp = app
        .oneshot(get_json("/api/servers"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn invalid_jwt_returns_401() {
    let app = test_app().await;
    let resp = app
        .oneshot(get_json_auth("/api/servers", "garbage"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn expired_jwt_returns_401() {
    use jsonwebtoken::{encode, EncodingKey, Header};
    let state = test_state().await;
    let app = test_app_with_state(state.clone());

    // Craft an expired JWT
    let claims = crate::handlers::auth::Claims {
        sub: "admin".to_string(),
        exp: 0, // epoch = expired
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&state.jwt_secret),
    )
    .unwrap();

    let resp = app
        .oneshot(get_json_auth("/api/servers", &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn valid_jwt_succeeds() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let resp = app
        .oneshot(get_json_auth("/api/servers", &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(json["servers"].is_array());
}

// ========== Logout ==========

#[tokio::test]
async fn logout_invalidates_refresh_tokens() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, refresh) = do_setup(&app).await;

    // Logout
    let resp = app
        .clone()
        .oneshot(post_json_auth("/api/auth/logout", &json!({}), &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Now the refresh token should be invalid
    let body = json!({"refresh_token": refresh});
    let resp = app
        .oneshot(post_json("/api/auth/refresh", &body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ========== Server-scoped 503s (no MatrixClient) ==========

#[tokio::test]
async fn command_no_client_returns_503() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let body = json!({"command": "server uptime"});
    let resp = app
        .oneshot(post_json_auth("/api/servers/999/command", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn list_users_no_client_returns_503() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let resp = app
        .oneshot(get_json_auth("/api/servers/999/users", &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn list_rooms_no_client_returns_503() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let resp = app
        .oneshot(get_json_auth("/api/servers/999/rooms", &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn server_status_no_client_returns_503() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let resp = app
        .oneshot(get_json_auth("/api/servers/999/server/status", &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn server_uptime_no_client_returns_503() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let resp = app
        .oneshot(get_json_auth("/api/servers/999/server/uptime", &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}

// ========== Input validation at HTTP layer ==========

#[tokio::test]
async fn command_rejects_control_chars() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let body = json!({"command": "server\x00uptime"});
    let resp = app
        .oneshot(post_json_auth("/api/servers/999/command", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn command_rejects_invalid_command() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let body = json!({"command": "nonexistent foo"});
    let resp = app
        .oneshot(post_json_auth("/api/servers/999/command", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_user_validates_username() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let body = json!({"username": "", "password": "validpass123"});
    let resp = app
        .oneshot(post_json_auth("/api/servers/999/users", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_user_validates_password() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let body = json!({"username": "newuser", "password": "short"});
    let resp = app
        .oneshot(post_json_auth("/api/servers/999/users", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ========== CSRF origin validation ==========

#[cfg(test)]
mod csrf_tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn csrf_blocks_mismatched_origin() {
        unsafe { std::env::set_var("CORS_ORIGIN", "https://allowed.example.com") };
        let state = test_state().await;
        let app = test_app_with_state(state);

        let req = Request::builder()
            .method("POST")
            .uri("/api/auth/status")
            .header("content-type", "application/json")
            .header("origin", "https://evil.example.com")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        unsafe { std::env::remove_var("CORS_ORIGIN") };
    }

    #[tokio::test]
    #[serial]
    async fn csrf_allows_matching_origin() {
        unsafe { std::env::set_var("CORS_ORIGIN", "https://allowed.example.com") };
        let state = test_state().await;
        let app = test_app_with_state(state);

        let body = json!({"username": "admin", "password": "supersecret123"});
        let req = Request::builder()
            .method("POST")
            .uri("/api/auth/setup")
            .header("content-type", "application/json")
            .header("origin", "https://allowed.example.com")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        // Should pass CSRF check (may be 200 or another status, but NOT 403 from origin check)
        assert_ne!(resp.status(), StatusCode::FORBIDDEN);
        unsafe { std::env::remove_var("CORS_ORIGIN") };
    }

    #[tokio::test]
    #[serial]
    async fn csrf_allows_no_origin_header() {
        unsafe { std::env::set_var("CORS_ORIGIN", "https://allowed.example.com") };
        let state = test_state().await;
        let app = test_app_with_state(state);

        let body = json!({"username": "admin", "password": "supersecret123"});
        let req = Request::builder()
            .method("POST")
            .uri("/api/auth/setup")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::FORBIDDEN);
        unsafe { std::env::remove_var("CORS_ORIGIN") };
    }

    #[tokio::test]
    #[serial]
    async fn csrf_allows_when_cors_unset() {
        unsafe { std::env::remove_var("CORS_ORIGIN") };
        let state = test_state().await;
        let app = test_app_with_state(state);

        let body = json!({"username": "admin", "password": "supersecret123"});
        let req = Request::builder()
            .method("POST")
            .uri("/api/auth/setup")
            .header("content-type", "application/json")
            .header("origin", "https://anything.example.com")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::FORBIDDEN);
    }
}
