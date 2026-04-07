use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::auth::jwt;
use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::state::AppState;
use vibetwitch_db::models::user::UserPublic;
use vibetwitch_db::queries;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserPublic,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/me", get(me))
}

async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    if body.username.len() < 3 || body.username.len() > 32 {
        return Err(AppError::BadRequest(
            "Username must be 3-32 characters".into(),
        ));
    }
    if body.password.len() < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters".into(),
        ));
    }

    // Check if username or email already taken
    if queries::users::find_by_email(&state.db, &body.email)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("Email already registered".into()));
    }
    if queries::users::find_by_username(&state.db, &body.username)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("Username already taken".into()));
    }

    let password_hash = hash_password(&body.password)?;

    let user =
        queries::users::create_user(&state.db, &body.username, &body.display_name, &body.email, &password_hash)
            .await?;

    let token =
        jwt::create_token(user.id, &state.config.jwt_secret).map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = queries::users::find_by_email(&state.db, &body.email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !verify_password(&body.password, &user.password_hash)? {
        return Err(AppError::Unauthorized);
    }

    let token =
        jwt::create_token(user.id, &state.config.jwt_secret).map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<UserPublic>, AppError> {
    let user = queries::users::find_by_id(&state.db, auth.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(user.into()))
}

fn hash_password(password: &str) -> Result<String, AppError> {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(hash.to_string())
}

fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    use argon2::{
        password_hash::PasswordHash, Argon2, PasswordVerifier,
    };

    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
