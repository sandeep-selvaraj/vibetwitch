use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Stream {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    #[serde(skip_serializing)]
    pub stream_key: String,
    pub hls_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub tags: Vec<String>,
    pub viewer_count: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StreamPublic {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub hls_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub tags: Vec<String>,
    pub viewer_count: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}
