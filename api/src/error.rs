use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub enum ApiError {
    NotConnected,
    MatrixError(String),
    Timeout,
    DbError(String),
    InvalidCommand(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NotConnected => write!(f, "Not connected to homeserver"),
            ApiError::MatrixError(msg) => write!(f, "Matrix error: {msg}"),
            ApiError::Timeout => write!(f, "Timed out waiting for response"),
            ApiError::DbError(msg) => write!(f, "Database error: {msg}"),
            ApiError::InvalidCommand(msg) => write!(f, "Invalid command: {msg}"),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::NotConnected => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            ApiError::MatrixError(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            ApiError::Timeout => (StatusCode::GATEWAY_TIMEOUT, self.to_string()),
            ApiError::DbError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::InvalidCommand(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        match &self {
            ApiError::InvalidCommand(_) => tracing::warn!("{message}"),
            _ => tracing::error!("{message}"),
        }
        let body = serde_json::json!({ "error": message });
        (status, axum::Json(body)).into_response()
    }
}
