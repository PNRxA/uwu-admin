use axum::Json;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::db;
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
        .checked_add_signed(chrono::Duration::days(30))
        .expect("valid timestamp")
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

// Extractor for authenticated requests
pub struct AuthUser {
    pub username: String,
}

impl FromRequestParts<SharedState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &SharedState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(ApiError::Unauthorized)?;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(&state.jwt_secret),
            &Validation::default(),
        )
        .map_err(|_| ApiError::Unauthorized)?;

        Ok(AuthUser {
            username: token_data.claims.sub,
        })
    }
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
    let count = db::count_admin_users(&state.db).await?;
    if count > 0 {
        return Err(ApiError::Forbidden("Setup already completed".into()));
    }

    if req.username.trim().is_empty() || req.password.trim().is_empty() {
        return Err(ApiError::BadRequest("Username and password are required".into()));
    }

    let password_hash = hash_password(&req.password)?;
    db::create_admin_user(&state.db, &req.username, &password_hash).await?;

    let token = create_token(&req.username, &state.jwt_secret)?;
    Ok(Json(json!({ "token": token })))
}

pub async fn login(
    State(state): State<SharedState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<Value>, ApiError> {
    let user = db::find_admin_user_by_username(&state.db, &req.username)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    if !verify_password(&req.password, &user.password_hash)? {
        return Err(ApiError::Unauthorized);
    }

    let token = create_token(&req.username, &state.jwt_secret)?;
    Ok(Json(json!({ "token": token })))
}
