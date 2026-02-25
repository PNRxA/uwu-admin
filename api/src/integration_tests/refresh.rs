use super::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn refresh_valid_token() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (_, refresh) = do_setup(&app).await;

    let resp = app
        .oneshot(post_with_cookie("/api/auth/refresh", &json!({}), &refresh))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // New refresh token should be in Set-Cookie
    let new_cookie = extract_refresh_cookie(&resp);
    assert!(new_cookie.is_some(), "refresh response should set new cookie");

    let json = body_json(resp).await;
    assert!(json["token"].is_string());
    assert!(json.get("refresh_token").is_none(), "refresh_token should not be in JSON body");
}

#[tokio::test]
#[serial]
async fn refresh_token_single_use() {
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (_, refresh) = do_setup(&app).await;

    // First use should succeed
    let resp = app
        .clone()
        .oneshot(post_with_cookie("/api/auth/refresh", &json!({}), &refresh))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Second use should fail (single-use rotation)
    let resp = app
        .oneshot(post_with_cookie("/api/auth/refresh", &json!({}), &refresh))
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

    let resp = app
        .oneshot(post_with_cookie("/api/auth/refresh", &json!({}), raw_token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
