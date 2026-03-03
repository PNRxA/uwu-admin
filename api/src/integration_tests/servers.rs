use super::*;
use crate::services::matrix::{self, MatrixClient};
use serial_test::serial;
use std::sync::Arc;

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
    let app = test_app_with_state(state.clone());
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
    let server_id = i32::try_from(json["id"].as_i64().unwrap()).unwrap();

    let client_arc = {
        let clients = state.clients.lock().await;
        Arc::clone(clients.get(&server_id).unwrap())
    };
    let mut client = client_arc.lock().await;
    let result = client
        .execute_command("server uptime", server_id, &state.db)
        .await
        .unwrap();
    assert!(!result.response.is_empty());

    let ctx = client.redaction_context();
    drop(client);

    matrix::redact_command_pair(
        &ctx,
        &result.command_event_id,
        &result.response_event_id,
    )
    .await;
}

#[tokio::test]
#[serial]
async fn command_redaction() {
    let Some((homeserver, username, password, room_id)) = test_server_env() else { return };
    let state = test_state().await;

    let mut client = MatrixClient::login(&homeserver, &username, &password, &room_id)
        .await
        .unwrap();

    let server_id = db::save_server(
        &state.db,
        &client.homeserver,
        client.access_token(),
        &client.room_id,
        &client.user_id,
        client.since(),
        &state.encryption_key,
    )
    .await
    .unwrap();

    let result = client
        .execute_command("server uptime", server_id, &state.db)
        .await
        .unwrap();

    assert!(!result.command_event_id.is_empty());
    assert!(!result.response_event_id.is_empty());
    assert!(!result.response.is_empty());

    let ctx = client.redaction_context();

    matrix::redact_command_pair(
        &ctx,
        &result.command_event_id,
        &result.response_event_id,
    )
    .await;

    // Verify events are actually redacted by fetching them from the server
    for event_id in [&result.command_event_id, &result.response_event_id] {
        assert!(
            is_event_redacted(&ctx.http, &ctx.homeserver, &ctx.access_token, &ctx.room_id, event_id).await,
            "Expected redacted (empty) content for {event_id}"
        );
    }
}

#[tokio::test]
#[serial]
async fn command_via_http_handler() {
    let Some((homeserver, username, password, room_id)) = test_server_env() else { return };
    let state = test_state().await;
    let app = test_app_with_state(state.clone());
    let (token, _) = do_setup(&app).await;

    // Add server via HTTP
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
    let server_id = json["id"].as_i64().unwrap();

    let server_id_i32 = i32::try_from(server_id).unwrap();
    let client_arc = {
        let clients = state.clients.lock().await;
        Arc::clone(clients.get(&server_id_i32).unwrap())
    };

    // Execute command via HTTP handler (which spawns redaction internally)
    let cmd_body = json!({ "command": "server uptime" });
    let resp = app
        .oneshot(post_json_auth(
            &format!("/api/servers/{server_id}/command"),
            &cmd_body,
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert!(
        json["response"].as_str().is_some_and(|s| !s.is_empty()),
        "Expected non-empty response from command handler"
    );

    // Wait for the spawned redaction task to complete
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Verify the handler's spawned redaction worked by checking that the most
    // recent m.room.message events from our user in the room are redacted.
    let client = client_arc.lock().await;
    let ctx = client.redaction_context();
    let user_id = client.user_id.clone();
    drop(client);

    let unredacted = count_unredacted_messages(
        &ctx.http,
        &ctx.homeserver,
        &ctx.access_token,
        &ctx.room_id,
        &user_id,
    )
    .await;
    assert_eq!(
        unredacted, 0,
        "Expected all message events from our user to be redacted, but {unredacted} remain"
    );
}

