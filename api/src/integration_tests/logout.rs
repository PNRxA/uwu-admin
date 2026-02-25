use super::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn logout_invalidates_refresh_tokens() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, refresh) = do_setup(&app).await;

    // Logout — should clear cookie
    let resp = app
        .clone()
        .oneshot(post_json_auth("/api/auth/logout", &json!({}), &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify the Set-Cookie clears the refresh token (Max-Age=0)
    let cookie_header = resp.headers().get("set-cookie")
        .expect("logout should set cookie")
        .to_str()
        .unwrap();
    assert!(cookie_header.contains("Max-Age=0"), "logout should expire the cookie");

    // Now the refresh token should be invalid
    let resp = app
        .oneshot(post_with_cookie("/api/auth/refresh", &json!({}), &refresh))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
