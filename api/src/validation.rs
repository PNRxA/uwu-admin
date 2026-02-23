use crate::error::ApiError;

pub fn validate_username(username: &str) -> Result<(), ApiError> {
    if username.is_empty()
        || username.chars().all(char::is_whitespace)
        || username.chars().any(char::is_control)
    {
        return Err(ApiError::BadRequest(
            "Invalid username: must not be empty or contain whitespace/control characters".into(),
        ));
    }
    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), ApiError> {
    if password.is_empty()
        || password.chars().all(char::is_whitespace)
        || password.chars().any(char::is_control)
    {
        return Err(ApiError::BadRequest(
            "Invalid password: must not be empty or contain whitespace/control characters".into(),
        ));
    }
    Ok(())
}

pub fn validate_matrix_room_id(room_id: &str) -> Result<(), ApiError> {
    if !room_id.starts_with('!')
        || !room_id.contains(':')
        || room_id.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        return Err(ApiError::BadRequest(
            "Invalid room ID: must start with '!' and contain ':'".into(),
        ));
    }
    Ok(())
}
