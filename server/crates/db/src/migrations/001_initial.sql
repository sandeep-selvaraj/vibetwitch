CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username      VARCHAR(32) UNIQUE NOT NULL,
    display_name  VARCHAR(64) NOT NULL,
    email         VARCHAR(255) UNIQUE NOT NULL,
    avatar_url    TEXT,
    bio           TEXT,
    password_hash TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE streams (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id        UUID NOT NULL REFERENCES users(id),
    title          VARCHAR(200) NOT NULL,
    description    TEXT,
    status         VARCHAR(16) NOT NULL DEFAULT 'offline',
    stream_key     VARCHAR(64) UNIQUE NOT NULL,
    hls_url        TEXT,
    thumbnail_url  TEXT,
    tags           TEXT[] NOT NULL DEFAULT '{}',
    viewer_count   INTEGER NOT NULL DEFAULT 0,
    started_at     TIMESTAMPTZ,
    ended_at       TIMESTAMPTZ,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE chat_messages (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id   UUID NOT NULL REFERENCES streams(id),
    user_id     UUID NOT NULL REFERENCES users(id),
    content     TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_chat_stream_time ON chat_messages(stream_id, created_at);

CREATE TABLE code_events (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id   UUID NOT NULL REFERENCES streams(id),
    event_type  VARCHAR(32) NOT NULL,
    payload     JSONB NOT NULL,
    sequence    BIGINT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_events_stream_seq ON code_events(stream_id, sequence);

CREATE TABLE follows (
    follower_id UUID NOT NULL REFERENCES users(id),
    followed_id UUID NOT NULL REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (follower_id, followed_id)
);
