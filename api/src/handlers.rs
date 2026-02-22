use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::ApiError;
use crate::matrix::MatrixClient;
use crate::state::SharedState;

#[derive(Deserialize)]
pub struct ConnectRequest {
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

pub async fn connect(
    State(state): State<SharedState>,
    Json(req): Json<ConnectRequest>,
) -> Result<Json<Value>, ApiError> {
    let client = MatrixClient::login(&req.homeserver, &req.username, &req.password, &req.room_id)
        .await?;

    let user_id = client.user_id.clone();
    let homeserver = client.homeserver.clone();

    *state.client.lock().await = Some(client);

    Ok(Json(json!({
        "connected": true,
        "user_id": user_id,
        "homeserver": homeserver
    })))
}

pub async fn disconnect(State(state): State<SharedState>) -> Json<Value> {
    *state.client.lock().await = None;
    Json(json!({ "connected": false }))
}

pub async fn status(State(state): State<SharedState>) -> Json<Value> {
    let lock = state.client.lock().await;
    match lock.as_ref() {
        Some(client) => Json(json!({
            "connected": true,
            "homeserver": client.homeserver,
            "user_id": client.user_id
        })),
        None => Json(json!({
            "connected": false
        })),
    }
}

pub async fn command(
    State(state): State<SharedState>,
    Json(req): Json<CommandRequest>,
) -> Result<Json<Value>, ApiError> {
    let mut lock = state.client.lock().await;
    let client = lock.as_mut().ok_or(ApiError::NotConnected)?;

    let response = client.execute_command(&req.command).await?;
    Ok(Json(json!({ "response": response })))
}

pub async fn list_users(State(state): State<SharedState>) -> Result<Json<Value>, ApiError> {
    let mut lock = state.client.lock().await;
    let client = lock.as_mut().ok_or(ApiError::NotConnected)?;

    let response = client.execute_command("users list").await?;
    Ok(Json(json!({ "response": response })))
}

pub async fn create_user(
    State(state): State<SharedState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<Value>, ApiError> {
    let mut lock = state.client.lock().await;
    let client = lock.as_mut().ok_or(ApiError::NotConnected)?;

    let cmd = match req.password {
        Some(ref pw) => format!("users create {} {}", req.username, pw),
        None => format!("users create {}", req.username),
    };

    let response = client.execute_command(&cmd).await?;
    Ok(Json(json!({ "response": response })))
}

pub async fn list_rooms(State(state): State<SharedState>) -> Result<Json<Value>, ApiError> {
    let mut lock = state.client.lock().await;
    let client = lock.as_mut().ok_or(ApiError::NotConnected)?;

    let response = client.execute_command("rooms list").await?;
    Ok(Json(json!({ "response": response })))
}

pub async fn room_info(
    State(state): State<SharedState>,
    Path(room_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let mut lock = state.client.lock().await;
    let client = lock.as_mut().ok_or(ApiError::NotConnected)?;

    let response = client.execute_command(&format!("rooms info {room_id}")).await?;
    Ok(Json(json!({ "response": response })))
}

pub async fn server_status(State(state): State<SharedState>) -> Result<Json<Value>, ApiError> {
    let mut lock = state.client.lock().await;
    let client = lock.as_mut().ok_or(ApiError::NotConnected)?;

    let response = client.execute_command("server stats").await?;
    Ok(Json(json!({ "response": response })))
}

pub async fn server_uptime(State(state): State<SharedState>) -> Result<Json<Value>, ApiError> {
    let mut lock = state.client.lock().await;
    let client = lock.as_mut().ok_or(ApiError::NotConnected)?;

    let response = client.execute_command("server uptime").await?;
    Ok(Json(json!({ "response": response })))
}
