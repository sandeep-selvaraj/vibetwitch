use sqlx::PgPool;
use uuid::Uuid;

use crate::models::ChatMessage;

pub async fn insert_message(
    pool: &PgPool,
    stream_id: Uuid,
    user_id: Uuid,
    content: &str,
) -> Result<ChatMessage, sqlx::Error> {
    sqlx::query_as::<_, ChatMessage>(
        r#"
        INSERT INTO chat_messages (stream_id, user_id, content)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(stream_id)
    .bind(user_id)
    .bind(content)
    .fetch_one(pool)
    .await
}

/// Fetch recent chat messages for a stream (for backfill on connect).
pub async fn recent_messages(
    pool: &PgPool,
    stream_id: Uuid,
    limit: i64,
) -> Result<Vec<ChatMessageWithUser>, sqlx::Error> {
    sqlx::query_as::<_, ChatMessageWithUser>(
        r#"
        SELECT cm.id, cm.stream_id, cm.user_id, cm.content, cm.created_at,
               u.username, u.display_name
        FROM chat_messages cm
        JOIN users u ON cm.user_id = u.id
        WHERE cm.stream_id = $1
        ORDER BY cm.created_at DESC
        LIMIT $2
        "#,
    )
    .bind(stream_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct ChatMessageWithUser {
    pub id: Uuid,
    pub stream_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub username: String,
    pub display_name: String,
}
