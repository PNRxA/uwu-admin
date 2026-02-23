use super::*;
use serial_test::serial;

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
