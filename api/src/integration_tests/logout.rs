use super::*;
use serial_test::serial;

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
