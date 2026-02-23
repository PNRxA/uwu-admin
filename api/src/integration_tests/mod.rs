mod auth;
mod commands;
mod csrf;
mod jwt;
mod logout;
mod refresh;
mod servers;
mod service_unavailable;
mod validation;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::services::db;
use crate::state::{AppState, SharedState};

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

fn delete_json_auth(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("DELETE")
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

/// Read integration-test env vars. Returns `None` if any are missing,
/// allowing the test to skip gracefully without `#[ignore]`.
fn test_server_env() -> Option<(String, String, String, String)> {
    let homeserver = std::env::var("TEST_HOMESERVER").ok()?;
    let username = std::env::var("TEST_USERNAME").ok()?;
    let password = std::env::var("TEST_PASSWORD").ok()?;
    let room_id = std::env::var("TEST_ROOM_ID").ok()?;
    Some((homeserver, username, password, room_id))
}
