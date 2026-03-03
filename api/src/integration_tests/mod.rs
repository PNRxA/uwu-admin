mod auth;
mod commands;
mod csrf;
mod jwt;
mod logout;
mod refresh;
mod servers;
mod service_unavailable;
mod settings;
mod validation;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use zeroize::Zeroizing;

use crate::services::db;
use crate::state::{AppState, SharedState};

async fn test_state() -> SharedState {
    let db = db::init_db("sqlite::memory:")
        .await
        .expect("in-memory DB");
    let jwt_secret = Zeroizing::new(vec![0xAA; 32]);
    let encryption_key = Zeroizing::new(vec![0xBB; 32]);
    AppState::new(db, jwt_secret, encryption_key, false, "UWUADMIN-test".to_string())
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

fn put_json_auth(uri: &str, body: &Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(uri)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(serde_json::to_vec(body).unwrap()))
        .unwrap()
}

/// Extract the refresh_token value from a Set-Cookie header
fn extract_refresh_cookie(response: &axum::response::Response) -> Option<String> {
    response
        .headers()
        .get("set-cookie")?
        .to_str()
        .ok()?
        .split(';')
        .next()?
        .strip_prefix("refresh_token=")
        .map(|s| s.to_string())
}

fn post_with_cookie(uri: &str, body: &Value, refresh_token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .header("cookie", format!("refresh_token={refresh_token}"))
        .body(Body::from(serde_json::to_vec(body).unwrap()))
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
    let refresh_token = extract_refresh_cookie(&resp)
        .expect("setup response should include Set-Cookie with refresh_token");
    let json = body_json(resp).await;
    (
        json["token"].as_str().unwrap().to_string(),
        refresh_token,
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

/// Check whether a Matrix event has been redacted (content stripped empty).
async fn is_event_redacted(
    http: &reqwest::Client,
    homeserver: &str,
    access_token: &str,
    room_id: &str,
    event_id: &str,
) -> bool {
    let encoded_rid =
        percent_encoding::utf8_percent_encode(room_id, percent_encoding::NON_ALPHANUMERIC)
            .to_string();
    let encoded_eid =
        percent_encoding::utf8_percent_encode(event_id, percent_encoding::NON_ALPHANUMERIC)
            .to_string();
    let url = format!(
        "{homeserver}/_matrix/client/v3/rooms/{encoded_rid}/event/{encoded_eid}"
    );
    let resp: Value = http
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    resp.get("content")
        .and_then(|c| c.as_object())
        .is_some_and(|o| o.is_empty())
}

/// Count how many `m.room.message` events from `user_id` still have
/// non-empty content (i.e. were NOT redacted) among the last N events.
async fn count_unredacted_messages(
    http: &reqwest::Client,
    homeserver: &str,
    access_token: &str,
    room_id: &str,
    user_id: &str,
) -> u32 {
    let encoded_rid =
        percent_encoding::utf8_percent_encode(room_id, percent_encoding::NON_ALPHANUMERIC)
            .to_string();
    let url = format!(
        "{homeserver}/_matrix/client/v3/rooms/{encoded_rid}/messages?dir=b&limit=10"
    );
    let resp: Value = http
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let Some(events) = resp["chunk"].as_array() else {
        return 0;
    };

    events
        .iter()
        .filter(|e| {
            e["sender"].as_str() == Some(user_id)
                && e["type"].as_str() == Some("m.room.message")
                && e.get("content")
                    .and_then(|c| c.as_object())
                    .is_some_and(|o| !o.is_empty())
        })
        .count() as u32
}
