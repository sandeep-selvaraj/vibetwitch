use axum::extract::{Path, State};
use axum::routing::{get, patch};
use axum::{Json, Router};
use serde::Deserialize;

use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::state::AppState;
use vibetwitch_db::models::user::UserPublic;
use vibetwitch_db::queries;

#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/me", patch(update_profile))
        .route("/{username}", get(get_user))
}

async fn get_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<UserPublic>, AppError> {
    let user = queries::users::find_by_username(&state.db, &username)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(user.into()))
}

async fn update_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpdateProfileRequest>,
) -> Result<Json<UserPublic>, AppError> {
    let user = queries::users::update_profile(
        &state.db,
        auth.user_id,
        &body.display_name,
        body.bio.as_deref(),
        body.avatar_url.as_deref(),
    )
    .await?;

    Ok(Json(user.into()))
}
