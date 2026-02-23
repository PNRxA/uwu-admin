use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::auth::AuthUser;
use crate::commands::validate_command;
use crate::db;
use crate::error::ApiError;
use crate::matrix::MatrixClient;
use crate::state::SharedState;
use crate::validation;

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
    _user: AuthUser,
    Json(req): Json<AddServerRequest>,
) -> Result<Json<Value>, ApiError> {
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
    )
    .await?;

    state.clients.lock().await.insert(server_id, client);

    Ok(Json(json!({
        "id": server_id,
        "homeserver": homeserver,
        "user_id": user_id
    })))
}

pub async fn list_servers(
    State(state): State<SharedState>,
    _user: AuthUser,
) -> Result<Json<Value>, ApiError> {
    let servers = db::load_all_servers(&state.db).await?;
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
    _user: AuthUser,
    Path(server_id): Path<i32>,
) -> Result<Json<Value>, ApiError> {
    state.clients.lock().await.remove(&server_id);
    db::delete_server(&state.db, server_id).await?;
    Ok(Json(json!({ "removed": true })))
}

pub async fn command(
    State(state): State<SharedState>,
    _user: AuthUser,
    Path(server_id): Path<i32>,
    Json(req): Json<CommandRequest>,
) -> Result<Json<Value>, ApiError> {
    validate_command(&req.command).map_err(ApiError::InvalidCommand)?;

    let mut lock = state.clients.lock().await;
    let client = lock.get_mut(&server_id).ok_or(ApiError::NotConnected)?;

    let response = client.execute_command(&req.command, server_id, &state.db).await?;
    Ok(Json(json!({ "response": response })))
}

pub async fn list_users(
    State(state): State<SharedState>,
    _user: AuthUser,
    Path(server_id): Path<i32>,
) -> Result<Json<Value>, ApiError> {
    let mut lock = state.clients.lock().await;
    let client = lock.get_mut(&server_id).ok_or(ApiError::NotConnected)?;

    let response = client.execute_command("users list-users", server_id, &state.db).await?;
    Ok(Json(json!({ "response": response })))
}

pub async fn create_user(
    State(state): State<SharedState>,
    _user: AuthUser,
    Path(server_id): Path<i32>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<Value>, ApiError> {
    validation::validate_username(&req.username)?;
    if let Some(ref pw) = req.password {
        validation::validate_password(pw)?;
    }

    let mut lock = state.clients.lock().await;
    let client = lock.get_mut(&server_id).ok_or(ApiError::NotConnected)?;

    let cmd = match req.password {
        Some(ref pw) => format!("users create-user {} {}", req.username, pw),
        None => format!("users create-user {}", req.username),
    };

    let response = client.execute_command(&cmd, server_id, &state.db).await?;
    Ok(Json(json!({ "response": response })))
}

pub async fn list_rooms(
    State(state): State<SharedState>,
    _user: AuthUser,
    Path(server_id): Path<i32>,
) -> Result<Json<Value>, ApiError> {
    let mut lock = state.clients.lock().await;
    let client = lock.get_mut(&server_id).ok_or(ApiError::NotConnected)?;

    let response = client.execute_command("rooms list-rooms", server_id, &state.db).await?;
    Ok(Json(json!({ "response": response })))
}

pub async fn server_status(
    State(state): State<SharedState>,
    _user: AuthUser,
    Path(server_id): Path<i32>,
) -> Result<Json<Value>, ApiError> {
    let mut lock = state.clients.lock().await;
    let client = lock.get_mut(&server_id).ok_or(ApiError::NotConnected)?;

    let response = client
        .execute_command("server memory-usage", server_id, &state.db)
        .await?;
    Ok(Json(json!({ "response": response })))
}

pub async fn server_uptime(
    State(state): State<SharedState>,
    _user: AuthUser,
    Path(server_id): Path<i32>,
) -> Result<Json<Value>, ApiError> {
    let mut lock = state.clients.lock().await;
    let client = lock.get_mut(&server_id).ok_or(ApiError::NotConnected)?;

    let response = client
        .execute_command("server uptime", server_id, &state.db)
        .await?;
    Ok(Json(json!({ "response": response })))
}
