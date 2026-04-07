use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use vibetwitch_shared::events::CodeEvent;

pub async fn run(url: String, mut rx: mpsc::UnboundedReceiver<CodeEvent>) {
    loop {
        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                tracing::info!("Connected to VibeTwitch server");
                let (mut ws_tx, mut ws_rx) = ws_stream.split();

                // Spawn a task to read server messages (mostly just to detect disconnects)
                let mut read_task = tokio::spawn(async move {
                    while let Some(msg) = ws_rx.next().await {
                        match msg {
                            Ok(Message::Close(_)) => break,
                            Err(e) => {
                                tracing::warn!("WS read error: {e}");
                                break;
                            }
                            _ => {}
                        }
                    }
                });

                // Send events as they come
                loop {
                    tokio::select! {
                        event = rx.recv() => {
                            match event {
                                Some(event) => {
                                    let msg = serde_json::json!({
                                        "type": "code_event",
                                        "event": event,
                                    });
                                    if let Err(e) = ws_tx.send(Message::Text(msg.to_string().into())).await {
                                        tracing::warn!("Failed to send event: {e}");
                                        break;
                                    }
                                }
                                None => {
                                    tracing::info!("Event channel closed, shutting down");
                                    return;
                                }
                            }
                        }
                        _ = &mut read_task => {
                            tracing::warn!("Server disconnected");
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to connect: {e}");
            }
        }

        tracing::info!("Reconnecting in 3 seconds...");
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}
