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
