use sqlx::PgPool;
use uuid::Uuid;

use crate::models::CodeEventRow;

pub async fn insert_event(
    pool: &PgPool,
    stream_id: Uuid,
    event_type: &str,
    payload: &serde_json::Value,
    sequence: i64,
) -> Result<CodeEventRow, sqlx::Error> {
    sqlx::query_as::<_, CodeEventRow>(
        r#"
        INSERT INTO code_events (stream_id, event_type, payload, sequence)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(stream_id)
    .bind(event_type)
    .bind(payload)
    .bind(sequence)
    .fetch_one(pool)
    .await
}

pub async fn get_events(
    pool: &PgPool,
    stream_id: Uuid,
    after_sequence: i64,
    limit: i64,
) -> Result<Vec<CodeEventRow>, sqlx::Error> {
    sqlx::query_as::<_, CodeEventRow>(
        r#"
        SELECT * FROM code_events
        WHERE stream_id = $1 AND sequence > $2
        ORDER BY sequence ASC
        LIMIT $3
        "#,
    )
    .bind(stream_id)
    .bind(after_sequence)
    .bind(limit)
    .fetch_all(pool)
    .await
}
