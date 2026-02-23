use super::*;
use serial_test::serial;

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
