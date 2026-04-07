use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/events/{stream_id}", get(ws_handler))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(stream_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_viewer(socket, stream_id, state))
}

async fn handle_viewer(socket: WebSocket, stream_id: Uuid, state: AppState) {
    let (mut ws_tx, _ws_rx) = socket.split();

    let mut event_rx = state.event_hub.subscribe(stream_id);

    loop {
        match event_rx.recv().await {
            Ok(broadcast) => {
                let json = serde_json::json!({
                    "type": "code_event",
                    "event": broadcast.event,
                    "sequence": broadcast.sequence,
                });
                if ws_tx.send(Message::text(json.to_string())).await.is_err() {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!(stream_id = %stream_id, lagged = n, "Event viewer lagged");
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}
