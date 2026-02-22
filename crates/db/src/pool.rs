//! Postgres connection pool.

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::info;

use crate::DbError;

/// Type alias for the shared Postgres pool used across the whole application.
pub type DbPool = PgPool;

/// Create a new connection pool from the given `database_url`.
///
/// `max_connections` controls the pool ceiling.
pub async fn create_pool(database_url: &str, max_connections: u32) -> Result<DbPool, DbError> {
    info!("Connecting to database (max_connections={})", max_connections);
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await?;
    Ok(pool)
}

/// Run embedded SQLx migrations located in `./migrations` (relative to the
/// workspace root at build time).
pub async fn run_migrations(pool: &DbPool) -> Result<(), DbError> {
    info!("Running database migrations");
    sqlx::migrate!("../../migrations").run(pool).await?;
    Ok(())
}
