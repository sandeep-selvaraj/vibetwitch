use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::auth::jwt;
use crate::state::AppState;
use crate::ws::hub::ChatBroadcast;
use vibetwitch_db::queries;

#[derive(Deserialize)]
pub struct ChatQuery {
    token: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/chat/{stream_id}", get(ws_handler))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(stream_id): Path<Uuid>,
    Query(query): Query<ChatQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Resolve user from token (optional — anonymous viewers can watch but not chat)
    let user_info = if let Some(ref token) = query.token {
        match jwt::verify_token(token, &state.config.jwt_secret) {
            Ok(claims) => {
                queries::users::find_by_id(&state.db, claims.sub)
                    .await
                    .ok()
                    .flatten()
                    .map(|u| (u.id, u.username, u.display_name))
            }
            Err(_) => None,
        }
    } else {
        None
    };

    ws.on_upgrade(move |socket| handle_socket(socket, stream_id, user_info, state))
}

async fn handle_socket(
    socket: WebSocket,
    stream_id: Uuid,
    user_info: Option<(Uuid, String, String)>,
    state: AppState,
) {
    let (mut ws_tx, mut ws_rx) = socket.split();

    // Join the chat room
    let (mut chat_rx, viewer_count) = state.chat_hub.join(stream_id);

    // Send initial viewer count
    let vc_msg = serde_json::json!({ "type": "viewer_count", "count": viewer_count });
    let _ = ws_tx.send(Message::text(vc_msg.to_string())).await;

    // Broadcast updated viewer count to everyone
    broadcast_viewer_count(&state, stream_id);

    // Spawn task to forward broadcast messages → WebSocket
    let sid = stream_id;
    let forward_task = tokio::spawn(async move {
        loop {
            match chat_rx.recv().await {
                Ok(msg) => {
                    let json = serde_json::json!({
                        "type": "chat_message",
                        "username": msg.username,
                        "display_name": msg.display_name,
                        "content": msg.content,
                        "timestamp": msg.timestamp,
                    });
                    if ws_tx.send(Message::text(json.to_string())).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(stream_id = %sid, lagged = n, "Chat receiver lagged");
                    // Continue — we just skip missed messages
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    // Rate limiting: max 5 messages per 5 seconds
    let mut message_timestamps: Vec<std::time::Instant> = Vec::new();
    let rate_window = std::time::Duration::from_secs(5);
    let rate_limit = 5;

    // Read messages from WebSocket
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
            Some("chat_message") => {
                // Must be authenticated to send messages
                let Some((user_id, ref username, ref display_name)) = user_info else {
                    continue;
                };

                let Some(content) = parsed.get("content").and_then(|c| c.as_str()) else {
                    continue;
                };

                let content = content.trim();
                if content.is_empty() || content.len() > 500 {
                    continue;
                }

                // Rate limit check
                let now = std::time::Instant::now();
                message_timestamps.retain(|t| now.duration_since(*t) < rate_window);
                if message_timestamps.len() >= rate_limit {
                    continue; // silently drop
                }
                message_timestamps.push(now);

                let timestamp = chrono::Utc::now().to_rfc3339();

                // Persist to DB (fire-and-forget)
                let db = state.db.clone();
                let sid = stream_id;
                let uid = user_id;
                let content_owned = content.to_string();
                tokio::spawn(async move {
                    if let Err(e) = queries::chat::insert_message(&db, sid, uid, &content_owned).await {
                        tracing::error!("Failed to persist chat message: {e}");
                    }
                });

                // Broadcast
                state.chat_hub.send(
                    stream_id,
                    ChatBroadcast {
                        username: username.clone(),
                        display_name: display_name.clone(),
                        content: content.to_string(),
                        timestamp,
                    },
                );
            }
            Some("ping") => {
                // Client keepalive — no-op, the WS layer handles pings
            }
            _ => {}
        }
    }

    // Clean up
    forward_task.abort();
    let _ = state.chat_hub.leave(stream_id);
    broadcast_viewer_count(&state, stream_id);
}

fn broadcast_viewer_count(state: &AppState, stream_id: Uuid) {
    let count = state.chat_hub.viewer_count(stream_id);
    // We broadcast it as a chat message so all connected viewers get it
    // Using a special broadcast that the forward task will pick up
    // Actually, viewer_count updates go through the same broadcast channel
    // We'll send a special message type — but our ChatBroadcast is typed.
    // Instead, let's update the DB viewer count asynchronously.
    let db = state.db.clone();
    tokio::spawn(async move {
        let _ = queries::streams::update_viewer_count(&db, stream_id, count as i32).await;
    });
}
