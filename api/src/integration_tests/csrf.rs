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
