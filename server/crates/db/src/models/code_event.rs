use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CodeEventRow {
    pub id: Uuid,
    pub stream_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub sequence: i64,
    pub created_at: DateTime<Utc>,
}
