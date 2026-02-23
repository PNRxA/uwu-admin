use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::Mutex;

use crate::services::commands::validate_command;
use crate::services::db;
use crate::error::ApiError;
use crate::services::matrix::MatrixClient;
use crate::services::response;
use crate::state::SharedState;
use crate::services::validation;

#[derive(Deserialize)]
pub struct AddServerRequest {
    homeserver: String,
    username: String,
    password: String,
    room_id: String,
}

#[derive(Deserialize)]
pub struct CommandRequest {
    command: String,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    username: String,
    password: Option<String>,
}

pub async fn add_server(
    State(state): State<SharedState>,
    Json(req): Json<AddServerRequest>,
) -> Result<Json<Value>, ApiError> {
    validation::validate_homeserver_url(&req.homeserver)?;

    let client = MatrixClient::login(&req.homeserver, &req.username, &req.password, &req.room_id)
        .await?;

    let user_id = client.user_id.clone();
    let homeserver = client.homeserver.clone();

    let server_id = db::save_server(
        &state.db,
        &client.homeserver,
        client.access_token(),
        &client.room_id,
        &client.user_id,
        client.since(),
        &state.encryption_key,
    )
    .await?;

    state.clients.lock().await.insert(server_id, Arc::new(Mutex::new(client)));

    tracing::info!(
        "Server added: id={}, homeserver={}, user_id={}",
        server_id, homeserver, user_id
    );

    Ok(Json(json!({
        "id": server_id,
        "homeserver": homeserver,
        "user_id": user_id
    })))
}

pub async fn list_servers(
    State(state): State<SharedState>,
) -> Result<Json<Value>, ApiError> {
    let servers = db::load_all_servers(&state.db, &state.encryption_key).await?;
    let clients = state.clients.lock().await;

    let list: Vec<Value> = servers
        .iter()
        .map(|s| {
            json!({
                "id": s.id,
                "homeserver": s.homeserver,
                "user_id": s.user_id,
                "connected": clients.contains_key(&s.id)
            })
        })
        .collect();

    Ok(Json(json!({ "servers": list })))
}

pub async fn remove_server(
    State(state): State<SharedState>,
    Path(server_id): Path<i32>,
) -> Result<Json<Value>, ApiError> {
    state.clients.lock().await.remove(&server_id);
    db::delete_server(&state.db, server_id).await?;
    tracing::info!("Server removed: id={}", server_id);
    Ok(Json(json!({ "removed": true })))
}

pub async fn command(
    State(state): State<SharedState>,
    Path(server_id): Path<i32>,
    Json(req): Json<CommandRequest>,
) -> Result<Json<Value>, ApiError> {
    if req.command.chars().any(char::is_control) {
        return Err(ApiError::BadRequest("Command must not contain control characters".into()));
    }
    validate_command(&req.command).map_err(ApiError::InvalidCommand)?;

    tracing::info!("Command executed: server_id={}, command='{}'", server_id, req.command);

    let client = {
        let lock = state.clients.lock().await;
        Arc::clone(lock.get(&server_id).ok_or(ApiError::NotConnected)?)
    };
    let mut client = client.lock().await;
    let raw = client.execute_command(&req.command, server_id, &state.db).await?;
    let parsed = response::parse_response(&raw);
    Ok(Json(json!({ "response": raw, "parsed": parsed })))
}

pub async fn list_users(
    State(state): State<SharedState>,
    Path(server_id): Path<i32>,
) -> Result<Json<Value>, ApiError> {
    let client = {
        let lock = state.clients.lock().await;
        Arc::clone(lock.get(&server_id).ok_or(ApiError::NotConnected)?)
    };
    let mut client = client.lock().await;
    let raw = client.execute_command("users list-users", server_id, &state.db).await?;
    let parsed = response::parse_response(&raw);
    Ok(Json(json!({ "response": raw, "parsed": parsed })))
}

pub async fn create_user(
    State(state): State<SharedState>,
    Path(server_id): Path<i32>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<Value>, ApiError> {
    validation::validate_username(&req.username)?;
    if let Some(ref pw) = req.password {
        validation::validate_password(pw)?;
        if pw.chars().any(char::is_whitespace) {
            return Err(ApiError::BadRequest(
                "Password for Matrix user creation must not contain spaces (protocol limitation)".into(),
            ));
        }
    }

    validation::validate_command_arg(&req.username, "username")?;
    if let Some(ref pw) = req.password {
        validation::validate_command_arg(pw, "password")?;
    }

    let client = {
        let lock = state.clients.lock().await;
        Arc::clone(lock.get(&server_id).ok_or(ApiError::NotConnected)?)
    };
    let mut client = client.lock().await;

    let cmd = match req.password {
        Some(ref pw) => format!("users create-user {} {}", req.username, pw),
        None => format!("users create-user {}", req.username),
    };

    tracing::info!(
        "User creation command sent: server_id={}, username='{}'",
        server_id, req.username
    );

    let raw = client.execute_command(&cmd, server_id, &state.db).await?;
    let parsed = response::parse_response(&raw);
    Ok(Json(json!({ "response": raw, "parsed": parsed })))
}

pub async fn list_rooms(
    State(state): State<SharedState>,
    Path(server_id): Path<i32>,
) -> Result<Json<Value>, ApiError> {
    let client = {
        let lock = state.clients.lock().await;
        Arc::clone(lock.get(&server_id).ok_or(ApiError::NotConnected)?)
    };
    let mut client = client.lock().await;
    let raw = client.execute_command("rooms list-rooms", server_id, &state.db).await?;
    let parsed = response::parse_response(&raw);
    Ok(Json(json!({ "response": raw, "parsed": parsed })))
}

pub async fn server_status(
    State(state): State<SharedState>,
    Path(server_id): Path<i32>,
) -> Result<Json<Value>, ApiError> {
    let client = {
        let lock = state.clients.lock().await;
        Arc::clone(lock.get(&server_id).ok_or(ApiError::NotConnected)?)
    };
    let mut client = client.lock().await;
    let raw = client
        .execute_command("server memory-usage", server_id, &state.db)
        .await?;
    let parsed = response::parse_response(&raw);
    Ok(Json(json!({ "response": raw, "parsed": parsed })))
}

pub async fn server_uptime(
    State(state): State<SharedState>,
    Path(server_id): Path<i32>,
) -> Result<Json<Value>, ApiError> {
    let client = {
        let lock = state.clients.lock().await;
        Arc::clone(lock.get(&server_id).ok_or(ApiError::NotConnected)?)
    };
    let mut client = client.lock().await;
    let raw = client
        .execute_command("server uptime", server_id, &state.db)
        .await?;
    let parsed = response::parse_response(&raw);
    Ok(Json(json!({ "response": raw, "parsed": parsed })))
}
