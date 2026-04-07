use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "payload")]
pub enum CodeEvent {
    #[serde(rename = "ai_message")]
    AiMessage(AiMessagePayload),
    #[serde(rename = "ai_response")]
    AiResponse(AiResponsePayload),
    #[serde(rename = "tool_use")]
    ToolUse(ToolUsePayload),
    #[serde(rename = "code_diff")]
    CodeDiff(CodeDiffPayload),
    #[serde(rename = "file_change")]
    FileChange(FileChangePayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessagePayload {
    pub tool: String,
    pub content: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponsePayload {
    pub tool: String,
    pub content: String,
    pub model: Option<String>,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsePayload {
    pub tool: String,
    pub tool_name: String,
    pub file_path: Option<String>,
    pub summary: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDiffPayload {
    pub file_path: String,
    pub diff: String,
    pub language: Option<String>,
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangePayload {
    pub file_path: String,
    pub change_type: FileChangeType,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEventRecord {
    pub id: Uuid,
    pub stream_id: Uuid,
    pub event: CodeEvent,
    pub sequence: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
