use serde::{Deserialize, Serialize};

use crate::events::CodeEvent;

/// Messages sent over the chat WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChatClientMsg {
    #[serde(rename = "chat_message")]
    Message { content: String },
    #[serde(rename = "ping")]
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChatServerMsg {
    #[serde(rename = "chat_message")]
    Message {
        username: String,
        display_name: String,
        content: String,
        timestamp: String,
    },
    #[serde(rename = "viewer_count")]
    ViewerCount { count: u32 },
    #[serde(rename = "pong")]
    Pong,
}

/// Messages sent over the code events WebSocket (viewer side)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventServerMsg {
    #[serde(rename = "code_event")]
    CodeEvent { event: CodeEvent, sequence: i64 },
}

/// Messages sent by the streamer agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IngestClientMsg {
    #[serde(rename = "code_event")]
    CodeEvent { event: CodeEvent },
    #[serde(rename = "heartbeat")]
    Heartbeat,
}
