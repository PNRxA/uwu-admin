use std::net::IpAddr;

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

fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
        }
        IpAddr::V6(v6) => {
            // Check IPv4-mapped addresses (::ffff:x.x.x.x) against the V4 rules
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_private_ip(&IpAddr::V4(v4));
            }
            v6.is_loopback()
                || v6.is_unspecified()
                || (v6.segments()[0] & 0xfe00) == 0xfc00 // fc00::/7 (unique local)
                || (v6.segments()[0] & 0xffc0) == 0xfe80 // fe80::/10 (link-local)
        }
    }
}

pub async fn validate_homeserver_url_resolved(url: &str) -> Result<(), ApiError> {
    validate_homeserver_url(url)?;

    let allow_private = std::env::var("ALLOW_PRIVATE_HOMESERVERS")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    if allow_private {
        return Ok(());
    }

    // Extract authority (host:port) from URL
    let after_scheme = if url.starts_with("https://") {
        &url[8..]
    } else {
        &url[7..]
    };
    let authority = after_scheme.split('/').next().unwrap_or(after_scheme);

    // Ensure there's a port for lookup_host (default based on scheme)
    let lookup_addr = if authority.contains(':') {
        authority.to_string()
    } else if url.starts_with("https://") {
        format!("{authority}:443")
    } else {
        format!("{authority}:80")
    };

    let addrs = tokio::net::lookup_host(&lookup_addr)
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to resolve homeserver hostname: {e}")))?;

    for addr in addrs {
        if is_private_ip(&addr.ip()) {
            return Err(ApiError::BadRequest(
                "Homeserver resolves to a private/internal IP address".into(),
            ));
        }
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

    // --- is_private_ip ---

    #[test]
    fn private_ip_loopback_v4() {
        assert!(is_private_ip(&"127.0.0.1".parse().unwrap()));
        assert!(is_private_ip(&"127.0.0.2".parse().unwrap()));
    }

    #[test]
    fn private_ip_loopback_v6() {
        assert!(is_private_ip(&"::1".parse().unwrap()));
    }

    #[test]
    fn private_ip_rfc1918() {
        assert!(is_private_ip(&"10.0.0.1".parse().unwrap()));
        assert!(is_private_ip(&"172.16.0.1".parse().unwrap()));
        assert!(is_private_ip(&"192.168.1.1".parse().unwrap()));
    }

    #[test]
    fn private_ip_link_local() {
        assert!(is_private_ip(&"169.254.1.1".parse().unwrap()));
    }

    #[test]
    fn private_ip_unspecified() {
        assert!(is_private_ip(&"0.0.0.0".parse().unwrap()));
        assert!(is_private_ip(&"::".parse().unwrap()));
    }

    #[test]
    fn private_ip_v4_mapped_v6() {
        // ::ffff:127.0.0.1 must be caught as private
        assert!(is_private_ip(&"::ffff:127.0.0.1".parse().unwrap()));
        assert!(is_private_ip(&"::ffff:10.0.0.1".parse().unwrap()));
        assert!(is_private_ip(&"::ffff:192.168.1.1".parse().unwrap()));
    }

    #[test]
    fn public_ip_not_private() {
        assert!(!is_private_ip(&"8.8.8.8".parse().unwrap()));
        assert!(!is_private_ip(&"1.1.1.1".parse().unwrap()));
        assert!(!is_private_ip(&"::ffff:8.8.8.8".parse().unwrap()));
    }

    // --- validate_homeserver_url_resolved (async) ---

    #[tokio::test]
    #[serial_test::serial]
    async fn resolved_rejects_localhost() {
        unsafe { std::env::remove_var("ALLOW_PRIVATE_HOMESERVERS") };
        let result = validate_homeserver_url_resolved("http://localhost:8008").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn resolved_rejects_127_0_0_1() {
        unsafe { std::env::remove_var("ALLOW_PRIVATE_HOMESERVERS") };
        let result = validate_homeserver_url_resolved("http://127.0.0.1:8008").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn resolved_rejects_private_10() {
        unsafe { std::env::remove_var("ALLOW_PRIVATE_HOMESERVERS") };
        let result = validate_homeserver_url_resolved("http://10.0.0.1:8008").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn resolved_rejects_private_192_168() {
        unsafe { std::env::remove_var("ALLOW_PRIVATE_HOMESERVERS") };
        let result = validate_homeserver_url_resolved("http://192.168.1.1:8008").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn resolved_allows_private_with_env_var() {
        // SAFETY: test-only; no other threads read this env var concurrently
        unsafe { std::env::set_var("ALLOW_PRIVATE_HOMESERVERS", "true") };
        let result = validate_homeserver_url_resolved("http://127.0.0.1:8008").await;
        unsafe { std::env::remove_var("ALLOW_PRIVATE_HOMESERVERS") };
        assert!(result.is_ok());
    }
}
