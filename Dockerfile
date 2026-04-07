# Stage 1: Build React frontend
FROM node:22-alpine AS web-builder
WORKDIR /app/web
RUN npm install -g pnpm@10
COPY web/package.json web/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY web/ ./
RUN pnpm build

# Stage 2: Build Rust API
FROM rust:1.88-bookworm AS server-builder
WORKDIR /app/server
COPY server/ ./
RUN cargo build --release --bin vibetwitch-api

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl supervisor && rm -rf /var/lib/apt/lists/*

# Install MediaMTX
RUN curl -fsSL https://github.com/bluenviron/mediamtx/releases/download/v1.17.1/mediamtx_v1.17.1_linux_amd64.tar.gz \
    | tar xz -C /usr/local/bin mediamtx

WORKDIR /app

# Copy built artifacts
COPY --from=server-builder /app/server/target/release/vibetwitch-api ./vibetwitch-api
COPY --from=web-builder /app/web/dist ./static
COPY deploy/mediamtx-prod.yml ./mediamtx.yml
COPY deploy/supervisord.conf ./supervisord.conf

EXPOSE 8000 1935 8888

CMD ["supervisord", "-c", "/app/supervisord.conf"]
