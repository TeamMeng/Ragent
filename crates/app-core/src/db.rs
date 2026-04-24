//! PostgreSQL connection pool.

use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::str::FromStr;

use crate::config::AppConfig;

/// Create a connection pool with sensible defaults.
pub async fn create_pool(config: &AppConfig) -> Result<PgPool, sqlx::Error> {
    let options = PgConnectOptions::from_str(&config.database.url)?.disable_statement_logging();

    PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect_with(options)
        .await
}

/// Run database migrations.
pub async fn run_migrations(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::migrate!("../../migrations").run(pool).await?;
    Ok(())
}
