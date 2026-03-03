use super::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn get_settings_returns_defaults() {
    let app = test_app().await;
    let (token, _) = do_setup(&app).await;

    let resp = app
        .oneshot(get_json_auth("/api/settings", &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["redact_messages"], "true");
}

#[tokio::test]
#[serial]
async fn update_settings_persists_value() {
    let app = test_app().await;
    let (token, _) = do_setup(&app).await;

    let body = json!({ "redact_messages": "false" });
    let resp = app
        .clone()
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["redact_messages"], "false");

    // Verify persisted via GET
    let resp = app
        .oneshot(get_json_auth("/api/settings", &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["redact_messages"], "false");
}

#[tokio::test]
#[serial]
async fn update_settings_rejects_unknown_key() {
    let app = test_app().await;
    let (token, _) = do_setup(&app).await;

    let body = json!({ "unknown_key": "value" });
    let resp = app
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial]
async fn update_settings_upserts() {
    let app = test_app().await;
    let (token, _) = do_setup(&app).await;

    // Set to false
    let body = json!({ "redact_messages": "false" });
    let resp = app
        .clone()
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Set back to true (upsert)
    let body = json!({ "redact_messages": "true" });
    let resp = app
        .clone()
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["redact_messages"], "true");
}

#[tokio::test]
#[serial]
async fn update_settings_rejects_invalid_boolean() {
    let app = test_app().await;
    let (token, _) = do_setup(&app).await;

    let body = json!({ "redact_messages": "not_a_bool" });
    let resp = app
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial]
async fn settings_requires_auth() {
    let app = test_app().await;

    let resp = app
        .clone()
        .oneshot(get_json("/api/settings"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let body = json!({ "redact_messages": "false" });
    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/settings")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
