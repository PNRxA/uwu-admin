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
        // Log full details before redacting
        match &self {
            ApiError::InvalidCommand(_) | ApiError::Unauthorized | ApiError::BadRequest(_) => {
                tracing::warn!("{self}")
            }
            ApiError::Forbidden(_) => tracing::warn!("{self}"),
            _ => tracing::error!("{self}"),
        }

        let (status, message) = match &self {
            ApiError::NotConnected => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            ApiError::MatrixError(_) => (StatusCode::BAD_GATEWAY, "Matrix server error".to_string()),
            ApiError::Timeout => (StatusCode::GATEWAY_TIMEOUT, self.to_string()),
            ApiError::DbError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
            ApiError::InvalidCommand(_) => (StatusCode::BAD_REQUEST, "Invalid command".to_string()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiError::Forbidden(_) => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        let body = serde_json::json!({ "error": message });
        (status, axum::Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    fn status_of(err: ApiError) -> StatusCode {
        err.into_response().status()
    }

    #[test]
    fn not_connected_is_503() {
        assert_eq!(status_of(ApiError::NotConnected), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn matrix_error_is_502() {
        assert_eq!(status_of(ApiError::MatrixError("detail".into())), StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn timeout_is_504() {
        assert_eq!(status_of(ApiError::Timeout), StatusCode::GATEWAY_TIMEOUT);
    }

    #[test]
    fn db_error_is_500() {
        assert_eq!(status_of(ApiError::DbError("detail".into())), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn invalid_command_is_400() {
        assert_eq!(status_of(ApiError::InvalidCommand("bad".into())), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn unauthorized_is_401() {
        assert_eq!(status_of(ApiError::Unauthorized), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn forbidden_is_403() {
        assert_eq!(status_of(ApiError::Forbidden("nope".into())), StatusCode::FORBIDDEN);
    }

    #[test]
    fn bad_request_is_400() {
        assert_eq!(status_of(ApiError::BadRequest("oops".into())), StatusCode::BAD_REQUEST);
    }
}
