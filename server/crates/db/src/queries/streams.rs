use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Stream, stream::StreamPublic};

pub async fn create_stream(
    pool: &PgPool,
    user_id: Uuid,
    title: &str,
    description: Option<&str>,
    tags: &[String],
    stream_key: &str,
) -> Result<Stream, sqlx::Error> {
    sqlx::query_as::<_, Stream>(
        r#"
        INSERT INTO streams (user_id, title, description, tags, stream_key)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(title)
    .bind(description)
    .bind(tags)
    .bind(stream_key)
    .fetch_one(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Stream>, sqlx::Error> {
    sqlx::query_as::<_, Stream>("SELECT * FROM streams WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn find_by_user_id(pool: &PgPool, user_id: Uuid) -> Result<Option<Stream>, sqlx::Error> {
    sqlx::query_as::<_, Stream>("SELECT * FROM streams WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

pub async fn get_live_streams(pool: &PgPool, limit: i64) -> Result<Vec<StreamPublic>, sqlx::Error> {
    sqlx::query_as::<_, StreamPublic>(
        r#"
        SELECT s.id, s.user_id, s.title, s.description, s.status,
               s.hls_url, s.thumbnail_url, s.tags, s.viewer_count, s.started_at,
               u.username, u.display_name, u.avatar_url
        FROM streams s
        JOIN users u ON s.user_id = u.id
        WHERE s.status = 'live'
        ORDER BY s.viewer_count DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn update_stream(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
    title: &str,
    description: Option<&str>,
    tags: &[String],
) -> Result<Stream, sqlx::Error> {
    sqlx::query_as::<_, Stream>(
        r#"
        UPDATE streams
        SET title = $3, description = $4, tags = $5
        WHERE id = $1 AND user_id = $2
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(title)
    .bind(description)
    .bind(tags)
    .fetch_one(pool)
    .await
}

pub async fn set_live(
    pool: &PgPool,
    id: Uuid,
    hls_url: &str,
) -> Result<Stream, sqlx::Error> {
    sqlx::query_as::<_, Stream>(
        r#"
        UPDATE streams
        SET status = 'live', hls_url = $2, started_at = now(), ended_at = NULL
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(hls_url)
    .fetch_one(pool)
    .await
}

pub async fn set_offline(pool: &PgPool, id: Uuid) -> Result<Stream, sqlx::Error> {
    sqlx::query_as::<_, Stream>(
        r#"
        UPDATE streams
        SET status = 'offline', ended_at = now(), viewer_count = 0
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn find_by_stream_key(
    pool: &PgPool,
    stream_key: &str,
) -> Result<Option<Stream>, sqlx::Error> {
    sqlx::query_as::<_, Stream>("SELECT * FROM streams WHERE stream_key = $1")
        .bind(stream_key)
        .fetch_optional(pool)
        .await
}

pub async fn update_viewer_count(pool: &PgPool, stream_id: Uuid, count: i32) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE streams SET viewer_count = $2 WHERE id = $1")
        .bind(stream_id)
        .bind(count)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn find_by_username(pool: &PgPool, username: &str) -> Result<Option<StreamPublic>, sqlx::Error> {
    sqlx::query_as::<_, StreamPublic>(
        r#"
        SELECT s.id, s.user_id, s.title, s.description, s.status,
               s.hls_url, s.thumbnail_url, s.tags, s.viewer_count, s.started_at,
               u.username, u.display_name, u.avatar_url
        FROM streams s
        JOIN users u ON s.user_id = u.id
        WHERE u.username = $1
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}
