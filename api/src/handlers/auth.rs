use axum::Json;
use axum::Extension;
use axum::extract::State;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Sha256, Digest};

use crate::services::db;
use crate::error::ApiError;
use crate::state::SharedState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn hash_password(password: &str) -> Result<String, ApiError> {
    use argon2::password_hash::{PasswordHasher, SaltString};
    use argon2::Argon2;

    let salt = SaltString::generate(&mut rand::rngs::OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| ApiError::BadRequest(format!("Failed to hash password: {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, ApiError> {
    use argon2::password_hash::{PasswordHash, PasswordVerifier};
    use argon2::Argon2;

    let parsed = PasswordHash::new(hash)
        .map_err(|e| ApiError::BadRequest(format!("Invalid password hash: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

fn create_token(username: &str, secret: &[u8]) -> Result<String, ApiError> {
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::minutes(15))
        .ok_or_else(|| ApiError::BadRequest("Failed to compute token expiry".into()))?
        .timestamp() as usize;

    let claims = Claims {
        sub: username.to_owned(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|e| ApiError::BadRequest(format!("Failed to create token: {e}")))
}

fn generate_refresh_token() -> String {
    use rand::Rng;
    let bytes: Vec<u8> = (0..32).map(|_| rand::rngs::OsRng.r#gen::<u8>()).collect();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub(crate) fn hash_refresh_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

async fn issue_token_pair(
    db: &sea_orm::DatabaseConnection,
    jwt_secret: &[u8],
    user_id: i32,
    username: &str,
) -> Result<(String, String), ApiError> {
    let access_token = create_token(username, jwt_secret)?;
    let raw_refresh = generate_refresh_token();
    let token_hash = hash_refresh_token(&raw_refresh);
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .ok_or_else(|| ApiError::BadRequest("Failed to compute refresh token expiry".into()))?
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    db::create_refresh_token(db, user_id, &token_hash, &expires_at).await?;

    Ok((access_token, raw_refresh))
}

#[derive(Clone)]
pub struct AuthUser {
    pub username: String,
}

// --- Auth handlers ---

#[derive(Deserialize)]
pub struct SetupRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    refresh_token: String,
}

pub async fn auth_status(
    State(state): State<SharedState>,
) -> Result<Json<Value>, ApiError> {
    let count = db::count_admin_users(&state.db).await?;
    Ok(Json(json!({ "setup_required": count == 0 })))
}

pub async fn setup(
    State(state): State<SharedState>,
    Json(req): Json<SetupRequest>,
) -> Result<Json<Value>, ApiError> {
    if req.username.trim().is_empty() || req.password.trim().is_empty() {
        return Err(ApiError::BadRequest("Username and password are required".into()));
    }

    if req.password.len() < 8 {
        return Err(ApiError::BadRequest(
            "Password must be at least 8 characters".into(),
        ));
    }

    let password_hash = hash_password(&req.password)?;
    let user = db::create_first_admin_user(&state.db, &req.username, &password_hash).await?;

    tracing::info!("Admin account setup completed for user '{}'", req.username);

    let (token, refresh_token) = issue_token_pair(&state.db, &state.jwt_secret, user.id, &user.username).await?;
    Ok(Json(json!({ "token": token, "refresh_token": refresh_token })))
}

pub async fn login(
    State(state): State<SharedState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<Value>, ApiError> {
    let user = match db::find_admin_user_by_username(&state.db, &req.username).await? {
        Some(user) => user,
        None => {
            tracing::warn!("Login failed: unknown username");
            return Err(ApiError::Unauthorized);
        }
    };

    if !verify_password(&req.password, &user.password_hash)? {
        tracing::warn!("Login failed: incorrect password");
        return Err(ApiError::Unauthorized);
    }

    tracing::info!("Admin login successful for user '{}'", req.username);

    let (token, refresh_token) = issue_token_pair(&state.db, &state.jwt_secret, user.id, &user.username).await?;
    Ok(Json(json!({ "token": token, "refresh_token": refresh_token })))
}

pub async fn refresh(
    State(state): State<SharedState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<Value>, ApiError> {
    let token_hash = hash_refresh_token(&req.refresh_token);

    let stored = db::find_refresh_token_by_hash(&state.db, &token_hash)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    // Validate expiry
    let expires_at = chrono::NaiveDateTime::parse_from_str(&stored.expires_at, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| ApiError::DbError(format!("Invalid expiry format: {e}")))?;
    let expires_at_utc = expires_at.and_utc();
    if expires_at_utc < chrono::Utc::now() {
        db::delete_refresh_token(&state.db, stored.id).await?;
        return Err(ApiError::Unauthorized);
    }

    // Look up user
    let user = db::find_admin_user_by_id(&state.db, stored.user_id)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    // Issue new token pair BEFORE deleting old (avoids gap where no valid token exists)
    let (token, refresh_token) = issue_token_pair(&state.db, &state.jwt_secret, user.id, &user.username).await?;

    // Rotation: delete old refresh token (single-use)
    db::delete_refresh_token(&state.db, stored.id).await?;

    Ok(Json(json!({ "token": token, "refresh_token": refresh_token })))
}

pub async fn logout(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Value>, ApiError> {
    let user = db::find_admin_user_by_username(&state.db, &auth_user.username)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    db::delete_refresh_tokens_for_user(&state.db, user.id).await?;

    Ok(Json(json!({ "ok": true })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_verify_password_roundtrip() {
        let hash = hash_password("testpassword").unwrap();
        assert!(verify_password("testpassword", &hash).unwrap());
    }

    #[test]
    fn verify_wrong_password() {
        let hash = hash_password("correct").unwrap();
        assert!(!verify_password("wrong", &hash).unwrap());
    }

    #[test]
    fn hash_refresh_token_deterministic() {
        let a = hash_refresh_token("token123");
        let b = hash_refresh_token("token123");
        assert_eq!(a, b);
    }

    #[test]
    fn different_inputs_different_hashes() {
        let a = hash_refresh_token("token_a");
        let b = hash_refresh_token("token_b");
        assert_ne!(a, b);
    }
}
