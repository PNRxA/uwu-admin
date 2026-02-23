use super::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn server_add_and_list() {
    let Some((homeserver, username, password, room_id)) = test_server_env() else { return };
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let body = json!({
        "homeserver": homeserver,
        "username": username,
        "password": password,
        "room_id": room_id,
    });
    let resp = app
        .clone()
        .oneshot(post_json_auth("/api/servers", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(json["id"].is_number());

    let resp = app
        .oneshot(get_json_auth("/api/servers", &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    let servers = json["servers"].as_array().unwrap();
    assert!(!servers.is_empty());
}

#[tokio::test]
#[serial]
async fn server_remove() {
    let Some((homeserver, username, password, room_id)) = test_server_env() else { return };
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let body = json!({
        "homeserver": homeserver,
        "username": username,
        "password": password,
        "room_id": room_id,
    });
    let resp = app
        .clone()
        .oneshot(post_json_auth("/api/servers", &body, &token))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let server_id = json["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(delete_json_auth(
            &format!("/api/servers/{server_id}"),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["removed"], true);
}

#[tokio::test]
#[serial]
async fn server_execute_command() {
    let Some((homeserver, username, password, room_id)) = test_server_env() else { return };
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let body = json!({
        "homeserver": homeserver,
        "username": username,
        "password": password,
        "room_id": room_id,
    });
    let resp = app
        .clone()
        .oneshot(post_json_auth("/api/servers", &body, &token))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let server_id = json["id"].as_i64().unwrap();

    let cmd = json!({"command": "server uptime"});
    let resp = app
        .oneshot(post_json_auth(
            &format!("/api/servers/{server_id}/command"),
            &cmd,
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(json["response"].is_string());
}

#[tokio::test]
#[serial]
async fn server_list_users() {
    let Some((homeserver, username, password, room_id)) = test_server_env() else { return };
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let body = json!({
        "homeserver": homeserver,
        "username": username,
        "password": password,
        "room_id": room_id,
    });
    let resp = app
        .clone()
        .oneshot(post_json_auth("/api/servers", &body, &token))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let server_id = json["id"].as_i64().unwrap();

    let resp = app
        .oneshot(get_json_auth(
            &format!("/api/servers/{server_id}/users"),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(json["response"].is_string());
}

#[tokio::test]
#[serial]
async fn server_list_rooms() {
    let Some((homeserver, username, password, room_id)) = test_server_env() else { return };
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let body = json!({
        "homeserver": homeserver,
        "username": username,
        "password": password,
        "room_id": room_id,
    });
    let resp = app
        .clone()
        .oneshot(post_json_auth("/api/servers", &body, &token))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let server_id = json["id"].as_i64().unwrap();

    let resp = app
        .oneshot(get_json_auth(
            &format!("/api/servers/{server_id}/rooms"),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(json["response"].is_string());
}

#[tokio::test]
#[serial]
async fn server_status_and_uptime() {
    let Some((homeserver, username, password, room_id)) = test_server_env() else { return };
    let state = test_state().await;
    let app = test_app_with_state(state);
    let (token, _) = do_setup(&app).await;

    let body = json!({
        "homeserver": homeserver,
        "username": username,
        "password": password,
        "room_id": room_id,
    });
    let resp = app
        .clone()
        .oneshot(post_json_auth("/api/servers", &body, &token))
        .await
        .unwrap();
    let json = body_json(resp).await;
    let server_id = json["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(get_json_auth(
            &format!("/api/servers/{server_id}/server/status"),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json_status = body_json(resp).await;
    assert!(json_status["response"].is_string());

    let resp = app
        .oneshot(get_json_auth(
            &format!("/api/servers/{server_id}/server/uptime"),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json_uptime = body_json(resp).await;
    assert!(json_uptime["response"].is_string());
}
