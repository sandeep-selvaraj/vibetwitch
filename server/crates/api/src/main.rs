mod auth;
mod config;
mod error;
mod routes;
mod state;
mod ws;

use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    dotenvy::dotenv().ok();

    let config = config::Config::from_env();
    let pool = vibetwitch_db::create_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    vibetwitch_db::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    let state = state::AppState::new(pool, config.clone());

    // Serve frontend static files with SPA fallback
    let spa = ServeDir::new("static").fallback(ServeFile::new("static/index.html"));

    let app = Router::new()
        .nest("/api", routes::router())
        .nest("/ws", ws::chat::router()
            .merge(ws::events::router())
            .merge(ws::ingest::router())
        )
        .with_state(state)
        .fallback_service(spa)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::very_permissive(), // tighten for production
        );

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
