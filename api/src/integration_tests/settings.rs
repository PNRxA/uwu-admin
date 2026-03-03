use super::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn get_public_settings_returns_theme_and_flavour() {
    let app = test_app().await;

    // Public endpoint works without auth
    let resp = app
        .clone()
        .oneshot(get_json("/api/settings/public"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;

    // Should contain theme and flavour_text defaults
    assert!(json["theme"].is_string());
    assert_eq!(json["flavour_text"], "true");

    // Should NOT contain protected settings
    assert!(json.get("redact_messages").is_none());
}

#[tokio::test]
#[serial]
async fn get_public_settings_reflects_changes() {
    let app = test_app().await;
    let (token, _) = do_setup(&app).await;

    // Update theme via authenticated endpoint
    let new_theme = json!({"activeThemeId":"slate","customThemes":[]});
    let body = json!({ "theme": new_theme.to_string() });
    let resp = app
        .clone()
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Public endpoint should reflect the change (no auth needed)
    let resp = app
        .oneshot(get_json("/api/settings/public"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    let theme: Value = serde_json::from_str(json["theme"].as_str().unwrap()).unwrap();
    assert_eq!(theme["activeThemeId"], "slate");
}

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

#[tokio::test]
#[serial]
async fn theme_setting_round_trip() {
    let app = test_app().await;
    let (token, _) = do_setup(&app).await;

    // Verify default
    let resp = app
        .clone()
        .oneshot(get_json_auth("/api/settings", &token))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let theme_default: Value = serde_json::from_str(json["theme"].as_str().unwrap()).unwrap();
    assert_eq!(theme_default["activeThemeId"], "uwu");
    assert_eq!(theme_default["customThemes"], json!([]));

    // Update theme
    let new_theme = json!({"activeThemeId":"slate","customThemes":[{"id":"my-theme","name":"My Theme","hue":200,"chromaScale":0.8}]});
    let body = json!({ "theme": new_theme.to_string() });
    let resp = app
        .clone()
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify persisted
    let resp = app
        .oneshot(get_json_auth("/api/settings", &token))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let theme: Value = serde_json::from_str(json["theme"].as_str().unwrap()).unwrap();
    assert_eq!(theme["activeThemeId"], "slate");
    assert_eq!(theme["customThemes"][0]["name"], "My Theme");
}

#[tokio::test]
#[serial]
async fn theme_setting_rejects_invalid_json() {
    let app = test_app().await;
    let (token, _) = do_setup(&app).await;

    let body = json!({ "theme": "not json" });
    let resp = app
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial]
async fn theme_setting_rejects_missing_fields() {
    let app = test_app().await;
    let (token, _) = do_setup(&app).await;

    // Missing customThemes
    let body = json!({ "theme": r#"{"activeThemeId":"uwu"}"# });
    let resp = app
        .clone()
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Missing activeThemeId
    let body = json!({ "theme": r#"{"customThemes":[]}"# });
    let resp = app
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial]
async fn theme_setting_rejects_oversized() {
    let app = test_app().await;
    let (token, _) = do_setup(&app).await;

    // Create a theme value larger than 10KB
    let large_name = "x".repeat(11_000);
    let large_theme = format!(r#"{{"activeThemeId":"uwu","customThemes":[{{"name":"{large_name}"}}]}}"#);
    let body = json!({ "theme": large_theme });
    let resp = app
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial]
async fn theme_setting_rejects_malformed_custom_theme() {
    let app = test_app().await;
    let (token, _) = do_setup(&app).await;

    // Item is not an object
    let theme = r#"{"activeThemeId":"uwu","customThemes":["not-an-object"]}"#;
    let body = json!({ "theme": theme });
    let resp = app
        .clone()
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Missing id
    let theme = r#"{"activeThemeId":"uwu","customThemes":[{"name":"X","hue":100,"chromaScale":1.0}]}"#;
    let body = json!({ "theme": theme });
    let resp = app
        .clone()
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Empty name
    let theme = r#"{"activeThemeId":"uwu","customThemes":[{"id":"x","name":"","hue":100,"chromaScale":1.0}]}"#;
    let body = json!({ "theme": theme });
    let resp = app
        .clone()
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Hue out of range
    let theme = r#"{"activeThemeId":"uwu","customThemes":[{"id":"x","name":"X","hue":400,"chromaScale":1.0}]}"#;
    let body = json!({ "theme": theme });
    let resp = app
        .clone()
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // chromaScale out of range
    let theme = r#"{"activeThemeId":"uwu","customThemes":[{"id":"x","name":"X","hue":100,"chromaScale":2.0}]}"#;
    let body = json!({ "theme": theme });
    let resp = app
        .clone()
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Valid custom theme should succeed
    let theme = r#"{"activeThemeId":"custom","customThemes":[{"id":"custom","name":"My Theme","hue":200,"chromaScale":0.8}]}"#;
    let body = json!({ "theme": theme });
    let resp = app
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn flavour_text_toggle() {
    let app = test_app().await;
    let (token, _) = do_setup(&app).await;

    // Default is true
    let resp = app
        .clone()
        .oneshot(get_json_auth("/api/settings", &token))
        .await
        .unwrap();
    let json = body_json(resp).await;
    assert_eq!(json["flavour_text"], "true");

    // Set to false
    let body = json!({ "flavour_text": "false" });
    let resp = app
        .clone()
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["flavour_text"], "false");

    // Set back to true
    let body = json!({ "flavour_text": "true" });
    let resp = app
        .oneshot(put_json_auth("/api/settings", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["flavour_text"], "true");
}
