//! Ragent API server — axum-based HTTP + WebSocket backend.

mod middleware;
mod routes;

use anyhow::Result;
use app_core::{AppConfig, create_pool, run_migrations};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: PgPool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env and configuration
    let config = AppConfig::load().expect("Failed to load config");

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ragent=debug,tower_http=debug,axum=trace".into()),
        )
        .init();

    info!("Starting Ragent server...");

    // Database
    let db = create_pool(&config).await.expect("Failed to connect to database");
    run_migrations(&db).await.expect("Failed to run migrations");
    info!("Database connected and migrations applied");

    let state = Arc::new(AppState { config: config.clone(), db });

    // Build router
    let app = routes::create_router(state);

    let host = &config.server.host;
    let port = config.server.port;
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .expect("Failed to bind");

    info!(host, port, "Server listening on http://{host}:{port}");
    axum::serve(listener, app).await?;

    Ok(())
}
