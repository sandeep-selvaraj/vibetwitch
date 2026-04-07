use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;

use crate::state::AppState;
use vibetwitch_db::queries;

/// MediaMTX sends this payload when authenticating a publish/read request.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct MediaMtxAuthRequest {
    pub action: String, // "publish", "read", "playback"
    pub path: String,   // e.g., "live/vt_abc123"
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub ip: String,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub protocol: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/mediamtx-auth", post(mediamtx_auth))
}

async fn mediamtx_auth(
    State(state): State<AppState>,
    Json(body): Json<MediaMtxAuthRequest>,
) -> StatusCode {
    tracing::info!(
        action = %body.action,
        path = %body.path,
        ip = %body.ip,
        protocol = %body.protocol,
        "MediaMTX auth request"
    );

    match body.action.as_str() {
        "publish" => {
            // Extract stream key from path: "live/vt_abc123" -> "vt_abc123"
            let stream_key = body
                .path
                .strip_prefix("live/")
                .unwrap_or(&body.path);

            match queries::streams::find_by_stream_key(&state.db, stream_key).await {
                Ok(Some(stream)) => {
                    // Auto-set stream to live
                    let hls_url = format!("http://localhost:8888/live/{}/index.m3u8", stream_key);
                    if let Err(e) = queries::streams::set_live(&state.db, stream.id, &hls_url).await {
                        tracing::error!("Failed to set stream live: {e}");
                    } else {
                        tracing::info!(stream_id = %stream.id, "Stream is now live");
                    }
                    StatusCode::OK
                }
                Ok(None) => {
                    tracing::warn!(stream_key, "Invalid stream key");
                    StatusCode::UNAUTHORIZED
                }
                Err(e) => {
                    tracing::error!("DB error during auth: {e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        }
        // Allow all read/playback requests (viewers)
        "read" | "playback" => StatusCode::OK,
        _ => StatusCode::OK,
    }
}
