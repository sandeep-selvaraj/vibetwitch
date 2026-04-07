use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use futures::StreamExt;
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;
use vibetwitch_db::queries;
use vibetwitch_shared::events::CodeEvent;

#[derive(Deserialize)]
pub struct IngestQuery {
    /// The stream key for authentication.
    stream_key: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/ingest/{stream_id}", get(ws_handler))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(stream_id): Path<Uuid>,
    Query(query): Query<IngestQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Validate stream key
    let stream = queries::streams::find_by_id(&state.db, stream_id).await;

    let authorized = match stream {
        Ok(Some(ref s)) => s.stream_key == query.stream_key,
        _ => false,
    };

    if !authorized {
        // Return the upgrade anyway but immediately close — axum requires us to
        // return the upgrade response. We'll close in the handler.
        return ws.on_upgrade(move |mut socket| async move {
            let _ = socket
                .send(Message::Close(None))
                .await;
        });
    }

    tracing::info!(stream_id = %stream_id, "Agent connected for event ingest");
    ws.on_upgrade(move |socket| handle_ingest(socket, stream_id, state))
}

async fn handle_ingest(socket: WebSocket, stream_id: Uuid, state: AppState) {
    let (_ws_tx, mut ws_rx) = socket.split();

    while let Some(Ok(msg)) = ws_rx.next().await {
        let text = match msg {
            Message::Text(t) => t.to_string(),
            Message::Close(_) => break,
            _ => continue,
        };

        let parsed: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => continue,
        };

        match parsed.get("type").and_then(|t| t.as_str()) {
            Some("code_event") => {
                // Parse the nested event
                let Some(event_value) = parsed.get("event") else {
                    continue;
                };

                let event: CodeEvent = match serde_json::from_value(event_value.clone()) {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::warn!("Invalid code event: {e}");
                        continue;
                    }
                };

                // Publish to hub (assigns sequence, broadcasts to viewers)
                let seq = state.event_hub.publish(stream_id, event.clone());

                // Persist to DB asynchronously
                let db = state.db.clone();
                let ev = event_value.clone();
                let event_type = match &event {
                    CodeEvent::AiMessage(_) => "ai_message",
                    CodeEvent::AiResponse(_) => "ai_response",
                    CodeEvent::ToolUse(_) => "tool_use",
                    CodeEvent::CodeDiff(_) => "code_diff",
                    CodeEvent::FileChange(_) => "file_change",
                };
                tokio::spawn(async move {
                    if let Err(e) = queries::events::insert_event(
                        &db,
                        stream_id,
                        event_type,
                        &ev,
                        seq,
                    )
                    .await
                    {
                        tracing::error!("Failed to persist code event: {e}");
                    }
                });
            }
            Some("heartbeat") => {
                // Agent keepalive — no-op
            }
            _ => {}
        }
    }

    tracing::info!(stream_id = %stream_id, "Agent disconnected from event ingest");
}
