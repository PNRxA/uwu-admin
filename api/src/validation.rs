use crate::error::ApiError;

pub fn validate_username(username: &str) -> Result<(), ApiError> {
    if username.is_empty()
        || username.chars().any(char::is_whitespace)
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
        || password.len() < 8
        || password.chars().any(char::is_whitespace)
        || password.chars().any(char::is_control)
    {
        return Err(ApiError::BadRequest(
            "Invalid password: must be at least 8 characters and not contain whitespace/control characters".into(),
        ));
    }
    Ok(())
}

pub fn validate_homeserver_url(url: &str) -> Result<(), ApiError> {
    if url.is_empty() {
        return Err(ApiError::BadRequest("Homeserver URL must not be empty".into()));
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(ApiError::BadRequest(
            "Homeserver URL must start with http:// or https://".into(),
        ));
    }

    if url.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(ApiError::BadRequest(
            "Homeserver URL must not contain whitespace or control characters".into(),
        ));
    }

    let after_scheme = if url.starts_with("https://") {
        &url[8..]
    } else {
        &url[7..]
    };

    if after_scheme.is_empty() || after_scheme.starts_with('/') {
        return Err(ApiError::BadRequest(
            "Homeserver URL must contain a hostname".into(),
        ));
    }

    Ok(())
}
