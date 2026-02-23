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
        || password.chars().any(char::is_control)
    {
        return Err(ApiError::BadRequest(
            "Invalid password: must be at least 8 characters and not contain control characters".into(),
        ));
    }
    Ok(())
}

/// Validates a single command argument: rejects empty, whitespace, and control chars.
pub fn validate_command_arg(arg: &str, name: &str) -> Result<(), ApiError> {
    if arg.is_empty()
        || arg.chars().any(char::is_whitespace)
        || arg.chars().any(char::is_control)
    {
        return Err(ApiError::BadRequest(
            format!("Invalid {name}: must not be empty or contain whitespace/control characters"),
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- validate_username ---

    #[test]
    fn username_valid() {
        assert!(validate_username("alice").is_ok());
    }

    #[test]
    fn username_empty() {
        assert!(validate_username("").is_err());
    }

    #[test]
    fn username_whitespace() {
        assert!(validate_username("ali ce").is_err());
        assert!(validate_username("alice\t").is_err());
    }

    #[test]
    fn username_control_chars() {
        assert!(validate_username("alice\x00").is_err());
        assert!(validate_username("ali\x07ce").is_err());
    }

    // --- validate_password ---

    #[test]
    fn password_valid() {
        assert!(validate_password("strong password!").is_ok());
    }

    #[test]
    fn password_empty() {
        assert!(validate_password("").is_err());
    }

    #[test]
    fn password_too_short() {
        assert!(validate_password("short").is_err());
        assert!(validate_password("1234567").is_err());
    }

    #[test]
    fn password_exactly_eight() {
        assert!(validate_password("12345678").is_ok());
    }

    #[test]
    fn password_control_chars() {
        assert!(validate_password("password\x00safe").is_err());
    }

    // --- validate_command_arg ---

    #[test]
    fn command_arg_valid() {
        assert!(validate_command_arg("@user:host", "user").is_ok());
    }

    #[test]
    fn command_arg_empty() {
        assert!(validate_command_arg("", "arg").is_err());
    }

    #[test]
    fn command_arg_whitespace() {
        assert!(validate_command_arg("has space", "arg").is_err());
    }

    #[test]
    fn command_arg_control_chars() {
        assert!(validate_command_arg("val\x00ue", "arg").is_err());
    }

    // --- validate_homeserver_url ---

    #[test]
    fn homeserver_url_valid_https() {
        assert!(validate_homeserver_url("https://matrix.example.com").is_ok());
    }

    #[test]
    fn homeserver_url_valid_http() {
        assert!(validate_homeserver_url("http://localhost:8008").is_ok());
    }

    #[test]
    fn homeserver_url_empty() {
        assert!(validate_homeserver_url("").is_err());
    }

    #[test]
    fn homeserver_url_missing_scheme() {
        assert!(validate_homeserver_url("matrix.example.com").is_err());
    }

    #[test]
    fn homeserver_url_scheme_only() {
        assert!(validate_homeserver_url("https://").is_err());
    }

    #[test]
    fn homeserver_url_scheme_with_slash() {
        assert!(validate_homeserver_url("https:///path").is_err());
    }

    #[test]
    fn homeserver_url_whitespace() {
        assert!(validate_homeserver_url("https://example .com").is_err());
    }

    #[test]
    fn homeserver_url_control_chars() {
        assert!(validate_homeserver_url("https://example\x00.com").is_err());
    }
}
