use axum::extract::{Path, Query, State};
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::state::AppState;
use vibetwitch_db::models::stream::StreamPublic;
use vibetwitch_db::models::Stream;
use vibetwitch_db::queries;

#[derive(Deserialize)]
pub struct CreateStreamRequest {
    pub title: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct UpdateStreamRequest {
    pub title: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct StreamWithKey {
    #[serde(flatten)]
    pub stream: Stream,
    pub stream_key_visible: String,
}

#[derive(Serialize)]
pub struct GoLiveResponse {
    pub rtmp_url: String,
    pub stream_key: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_stream))
        .route("/live", get(get_live_streams))
        .route("/mine", get(get_my_stream))
        .route("/by-username/{username}", get(get_stream_by_username))
        .route("/{id}", get(get_stream))
        .route("/{id}", patch(update_stream))
        .route("/{id}/chat", get(get_chat_history))
        .route("/{id}/events", get(get_events))
        .route("/{id}/go-live", post(go_live))
        .route("/{id}/end", post(end_stream))
}

async fn create_stream(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateStreamRequest>,
) -> Result<Json<StreamWithKey>, AppError> {
    // Check if user already has a stream
    if queries::streams::find_by_user_id(&state.db, auth.user_id)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict(
            "You already have a stream configured. Use PATCH to update it.".into(),
        ));
    }

    let stream_key = generate_stream_key();
    let tags = body.tags.unwrap_or_default();

    let stream = queries::streams::create_stream(
        &state.db,
        auth.user_id,
        &body.title,
        body.description.as_deref(),
        &tags,
        &stream_key,
    )
    .await?;

    Ok(Json(StreamWithKey {
        stream_key_visible: stream.stream_key.clone(),
        stream,
    }))
}

async fn get_stream_by_username(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<StreamPublic>, AppError> {
    let stream = queries::streams::find_by_username(&state.db, &username)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(stream))
}

async fn get_stream(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Stream>, AppError> {
    let stream = queries::streams::find_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(stream))
}

async fn get_my_stream(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Option<StreamWithKey>>, AppError> {
    let stream = queries::streams::find_by_user_id(&state.db, auth.user_id).await?;

    Ok(Json(stream.map(|s| StreamWithKey {
        stream_key_visible: s.stream_key.clone(),
        stream: s,
    })))
}

async fn get_live_streams(
    State(state): State<AppState>,
) -> Result<Json<Vec<StreamPublic>>, AppError> {
    let streams = queries::streams::get_live_streams(&state.db, 50).await?;
    Ok(Json(streams))
}

async fn update_stream(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateStreamRequest>,
) -> Result<Json<Stream>, AppError> {
    let tags = body.tags.unwrap_or_default();
    let stream = queries::streams::update_stream(
        &state.db,
        id,
        auth.user_id,
        &body.title,
        body.description.as_deref(),
        &tags,
    )
    .await?;

    Ok(Json(stream))
}

async fn go_live(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<GoLiveResponse>, AppError> {
    let stream = queries::streams::find_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound)?;

    if stream.user_id != auth.user_id {
        return Err(AppError::Unauthorized);
    }

    let hls_url = format!("{}/live/{}/index.m3u8", state.config.hls_base_url, stream.stream_key);
    queries::streams::set_live(&state.db, id, &hls_url).await?;

    Ok(Json(GoLiveResponse {
        rtmp_url: format!("{}/live/{}", state.config.rtmp_base_url, stream.stream_key),
        stream_key: stream.stream_key,
    }))
}

async fn end_stream(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Stream>, AppError> {
    let stream = queries::streams::find_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound)?;

    if stream.user_id != auth.user_id {
        return Err(AppError::Unauthorized);
    }

    let stream = queries::streams::set_offline(&state.db, id).await?;
    Ok(Json(stream))
}

async fn get_chat_history(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<vibetwitch_db::queries::chat::ChatMessageWithUser>>, AppError> {
    let messages = queries::chat::recent_messages(&state.db, id, 50).await?;
    // Messages come newest-first from DB, reverse for chronological order
    let mut messages = messages;
    messages.reverse();
    Ok(Json(messages))
}

#[derive(Deserialize)]
struct EventsQuery {
    #[serde(default)]
    after_sequence: i64,
    #[serde(default = "default_events_limit")]
    limit: i64,
}

fn default_events_limit() -> i64 {
    100
}

async fn get_events(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<EventsQuery>,
) -> Result<Json<Vec<vibetwitch_db::models::CodeEventRow>>, AppError> {
    let events = queries::events::get_events(
        &state.db,
        id,
        query.after_sequence,
        query.limit.min(200),
    )
    .await?;
    Ok(Json(events))
}

fn generate_stream_key() -> String {
    use rand::distr::Alphanumeric;
    let key: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    format!("vt_{key}")
}
