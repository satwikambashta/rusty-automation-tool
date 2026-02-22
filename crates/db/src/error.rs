//! Typed error type for the db crate.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("row not found")]
    NotFound,

    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
}
