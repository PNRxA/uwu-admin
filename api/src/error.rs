use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub enum ApiError {
    NotConnected,
    MatrixError(String),
    Timeout,
    DbError(String),
    InvalidCommand(String),
    Unauthorized,
    Forbidden(String),
    BadRequest(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NotConnected => write!(f, "Not connected to homeserver"),
            ApiError::MatrixError(msg) => write!(f, "Matrix error: {msg}"),
            ApiError::Timeout => write!(f, "Timed out waiting for response"),
            ApiError::DbError(msg) => write!(f, "Database error: {msg}"),
            ApiError::InvalidCommand(msg) => write!(f, "Invalid command: {msg}"),
            ApiError::Unauthorized => write!(f, "Unauthorized"),
            ApiError::Forbidden(msg) => write!(f, "Forbidden: {msg}"),
            ApiError::BadRequest(msg) => write!(f, "Bad request: {msg}"),
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
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiError::Forbidden(_) => (StatusCode::FORBIDDEN, self.to_string()),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        match &self {
            ApiError::InvalidCommand(_) | ApiError::Unauthorized | ApiError::BadRequest(_) => {
                tracing::warn!("{message}")
            }
            ApiError::Forbidden(_) => tracing::warn!("{message}"),
            _ => tracing::error!("{message}"),
        }
        let body = serde_json::json!({ "error": message });
        (status, axum::Json(body)).into_response()
    }
}
