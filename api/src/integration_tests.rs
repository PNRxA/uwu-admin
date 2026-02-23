use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serial_test::serial;
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

// ========== Auth flow ==========

#[tokio::test]
#[serial]
async fn auth_status_setup_required() {
    let app = test_app().await;
    let resp = app.oneshot(get_json("/api/auth/status")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["setup_required"], true);
}

#[tokio::test]
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
async fn no_jwt_returns_401() {
    let app = test_app().await;
    let resp = app
        .oneshot(get_json("/api/servers"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn invalid_jwt_returns_401() {
    let app = test_app().await;
    let resp = app
        .oneshot(get_json_auth("/api/servers", "garbage"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
    async fn csrf_rejects_missing_origin_header() {
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
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
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

// ========== Exhaustive command-tree integration test ==========

#[derive(Debug)]
struct TestArg {
    name: String,
    arg_type: String,
    required: bool,
}

#[derive(Debug)]
struct TestNode {
    name: String,
    children: Vec<TestNode>,
    args: Vec<TestArg>,
}

fn parse_command_tree(value: &[Value]) -> Vec<TestNode> {
    value
        .iter()
        .map(|v| {
            let name = v["name"].as_str().unwrap_or("").to_string();
            let children = v
                .get("children")
                .and_then(|c| c.as_array())
                .map(|arr| parse_command_tree(arr))
                .unwrap_or_default();
            let args = v
                .get("args")
                .and_then(|a| a.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|a| TestArg {
                            name: a["name"].as_str().unwrap_or("").to_string(),
                            arg_type: a["type"].as_str().unwrap_or("string").to_string(),
                            required: a["required"].as_bool().unwrap_or(false),
                        })
                        .collect()
                })
                .unwrap_or_default();
            TestNode { name, children, args }
        })
        .collect()
}

fn collect_leaf_commands<'a>(
    nodes: &'a [TestNode],
    prefix: &str,
    out: &mut Vec<(String, Vec<&'a TestArg>)>,
) {
    for node in nodes {
        let path = if prefix.is_empty() {
            node.name.clone()
        } else {
            format!("{prefix} {}", node.name)
        };
        if node.children.is_empty() {
            let args: Vec<&TestArg> = node.args.iter().collect();
            out.push((path, args));
        } else {
            collect_leaf_commands(&node.children, &path, out);
        }
    }
}

#[tokio::test]
#[serial]
async fn execute_all_command_tree_commands() {
    let _ = dotenvy::dotenv();
    let Some((homeserver, username, password, room_id)) = test_server_env() else {
        eprintln!("Skipping execute_all_command_tree_commands: env vars not set");
        return;
    };

    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    // Add server
    let body = json!({
        "homeserver": homeserver,
        "username": username,
        "password": password,
        "room_id": room_id,
    });
    let resp = app
        .clone()
        .oneshot(post_json_auth("/api/servers", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    let server_id = json["id"].as_i64().unwrap();

    // Extract server name from room_id (part after ':')
    let server_name = room_id
        .split(':')
        .nth(1)
        .expect("TEST_ROOM_ID must contain ':'");

    // Create a test user
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let test_username = format!("uwu_test_{ts}");
    let test_user_id = format!("@{test_username}:{server_name}");

    let create_cmd = json!({"command": format!("users create-user {test_username}")});
    let resp = app
        .clone()
        .oneshot(post_json_auth(
            &format!("/api/servers/{server_id}/command"),
            &create_cmd,
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "Failed to create test user");

    // Parse command tree
    let tree_json: Vec<Value> =
        serde_json::from_str(include_str!("../../shared/command-tree.json"))
            .expect("Failed to parse command-tree.json");
    let tree = parse_command_tree(&tree_json);

    let mut leaves = Vec::new();
    collect_leaf_commands(&tree, "", &mut leaves);

    // Skip lists
    let destructive_skip: &[&str] = &[
        "server shutdown",
        "server restart",
        "server reload-mods",
        "users deactivate-all",
        "users make-user-admin",
        "rooms moderation ban-list-of-rooms",
    ];
    let code_block_skip: &[&str] = &[
        "appservices register",
        "debug parse-pdu",
        "debug verify-json",
        "debug get-remote-pdu-list",
        "media delete-list",
        "users force-join-list-of-local-users",
    ];

    let mut tested = 0u32;
    let mut skipped = 0u32;
    let mut failures: Vec<(String, String)> = Vec::new();

    for (path, args) in &leaves {
        if destructive_skip.contains(&path.as_str()) || code_block_skip.contains(&path.as_str()) {
            eprintln!("  SKIP: {path}");
            skipped += 1;
            continue;
        }

        // Build command string with arg values
        let mut cmd_string = path.clone();
        for arg in args {
            let val = match arg.arg_type.as_str() {
                "user_id" => test_user_id.clone(),
                "room_id" => room_id.clone(),
                "server" => server_name.to_string(),
                "number" => "1".to_string(),
                "event_id" => "$placeholder:test".to_string(),
                "path" => "/tmp/test".to_string(),
                _ => "test".to_string(), // string and any other type
            };
            cmd_string.push(' ');
            cmd_string.push_str(&val);
        }

        eprintln!("  RUN:  {cmd_string}");

        let cmd_body = json!({"command": cmd_string});
        let resp = app
            .clone()
            .oneshot(post_json_auth(
                &format!("/api/servers/{server_id}/command"),
                &cmd_body,
                &token,
            ))
            .await
            .unwrap();

        let status = resp.status();
        let body = body_json(resp).await;

        if status != StatusCode::OK {
            failures.push((path.clone(), format!("status={status}")));
        } else if !body["response"].is_string() {
            failures.push((path.clone(), "response is not a string".to_string()));
        } else if let Some(resp_text) = body["response"].as_str() {
            if resp_text.contains("error:") {
                failures.push((path.clone(), resp_text.to_string()));
            }
        }

        tested += 1;
    }

    // Cleanup: deactivate the test user
    let deactivate_cmd = json!({"command": format!("users deactivate {test_user_id}")});
    let _ = app
        .clone()
        .oneshot(post_json_auth(
            &format!("/api/servers/{server_id}/command"),
            &deactivate_cmd,
            &token,
        ))
        .await;

    // Cleanup: unban any rooms that may have been banned during the test
    let unban_cmd = json!({"command": format!("rooms moderation unban-room {room_id}")});
    let _ = app
        .clone()
        .oneshot(post_json_auth(
            &format!("/api/servers/{server_id}/command"),
            &unban_cmd,
            &token,
        ))
        .await;

    eprintln!("\n=== Command Tree Test Summary ===");
    eprintln!("  Tested:  {tested}");
    eprintln!("  Skipped: {skipped}");
    eprintln!("  Failed:  {}", failures.len());

    if !failures.is_empty() {
        for (path, reason) in &failures {
            eprintln!("  FAIL: {path} — {reason}");
        }
        panic!(
            "{} command(s) failed: {}",
            failures.len(),
            failures
                .iter()
                .map(|(p, r)| format!("{p} ({r})"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}

// ========== Server management (require a live homeserver) ==========

/// Read integration-test env vars. Returns `None` if any are missing,
/// allowing the test to skip gracefully without `#[ignore]`.
fn test_server_env() -> Option<(String, String, String, String)> {
    let homeserver = std::env::var("TEST_HOMESERVER").ok()?;
    let username = std::env::var("TEST_USERNAME").ok()?;
    let password = std::env::var("TEST_PASSWORD").ok()?;
    let room_id = std::env::var("TEST_ROOM_ID").ok()?;
    Some((homeserver, username, password, room_id))
}

#[tokio::test]
#[serial]
async fn server_add_and_list() {
    let Some((homeserver, username, password, room_id)) = test_server_env() else { return };
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let body = json!({
        "homeserver": homeserver,
        "username": username,
        "password": password,
        "room_id": room_id,
    });
    let resp = app
        .clone()
        .oneshot(post_json_auth("/api/servers", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(json["id"].is_number());

    let resp = app
        .oneshot(get_json_auth("/api/servers", &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    let servers = json["servers"].as_array().unwrap();
    assert!(!servers.is_empty());
}

#[tokio::test]
#[serial]
async fn server_remove() {
    let Some((homeserver, username, password, room_id)) = test_server_env() else { return };
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let body = json!({
        "homeserver": homeserver,
        "username": username,
        "password": password,
        "room_id": room_id,
    });
    let resp = app
        .clone()
        .oneshot(post_json_auth("/api/servers", &body, &token))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let server_id = json["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(delete_json_auth(
            &format!("/api/servers/{server_id}"),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["removed"], true);
}

#[tokio::test]
#[serial]
async fn server_execute_command() {
    let Some((homeserver, username, password, room_id)) = test_server_env() else { return };
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let body = json!({
        "homeserver": homeserver,
        "username": username,
        "password": password,
        "room_id": room_id,
    });
    let resp = app
        .clone()
        .oneshot(post_json_auth("/api/servers", &body, &token))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let server_id = json["id"].as_i64().unwrap();

    let cmd = json!({"command": "server uptime"});
    let resp = app
        .oneshot(post_json_auth(
            &format!("/api/servers/{server_id}/command"),
            &cmd,
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(json["response"].is_string());
}

#[tokio::test]
#[serial]
async fn server_list_users() {
    let Some((homeserver, username, password, room_id)) = test_server_env() else { return };
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let body = json!({
        "homeserver": homeserver,
        "username": username,
        "password": password,
        "room_id": room_id,
    });
    let resp = app
        .clone()
        .oneshot(post_json_auth("/api/servers", &body, &token))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let server_id = json["id"].as_i64().unwrap();

    let resp = app
        .oneshot(get_json_auth(
            &format!("/api/servers/{server_id}/users"),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(json["response"].is_string());
}

#[tokio::test]
#[serial]
async fn server_list_rooms() {
    let Some((homeserver, username, password, room_id)) = test_server_env() else { return };
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let body = json!({
        "homeserver": homeserver,
        "username": username,
        "password": password,
        "room_id": room_id,
    });
    let resp = app
        .clone()
        .oneshot(post_json_auth("/api/servers", &body, &token))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let server_id = json["id"].as_i64().unwrap();

    let resp = app
        .oneshot(get_json_auth(
            &format!("/api/servers/{server_id}/rooms"),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(json["response"].is_string());
}

#[tokio::test]
#[serial]
async fn server_status_and_uptime() {
    let Some((homeserver, username, password, room_id)) = test_server_env() else { return };
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let body = json!({
        "homeserver": homeserver,
        "username": username,
        "password": password,
        "room_id": room_id,
    });
    let resp = app
        .clone()
        .oneshot(post_json_auth("/api/servers", &body, &token))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let server_id = json["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(get_json_auth(
            &format!("/api/servers/{server_id}/server/status"),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json_status = body_json(resp).await;
    assert!(json_status["response"].is_string());

    let resp = app
        .oneshot(get_json_auth(
            &format!("/api/servers/{server_id}/server/uptime"),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json_uptime = body_json(resp).await;
    assert!(json_uptime["response"].is_string());
}
