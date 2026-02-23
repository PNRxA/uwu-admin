use super::*;
use serial_test::serial;

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
