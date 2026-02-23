use super::*;
use serial_test::serial;

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
