# VibeTwitch development task runner

# Start infrastructure (Postgres + Redis)
infra:
    docker compose up -d

# Stop infrastructure
infra-down:
    docker compose down

# Run the Rust API server
server:
    cd server && cargo run --bin vibetwitch-api

# Run the frontend dev server
web:
    cd web && pnpm dev

# Run both server and web in parallel
dev: infra
    #!/usr/bin/env bash
    trap 'kill 0' EXIT
    just server &
    just web &
    wait

# Check all code compiles
check:
    cd server && cargo check
    cd web && pnpm tsc --noEmit

# Build everything for production
build:
    cd server && cargo build --release
    cd web && pnpm build

# Run database migrations (via the server startup)
migrate:
    cd server && cargo run -- --migrate-only 2>/dev/null || echo "Migrations run on server start"

# Format code
fmt:
    cd server && cargo fmt
    cd web && pnpm prettier --write src/
