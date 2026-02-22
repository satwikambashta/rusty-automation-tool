//! `db` crate — pure persistence layer.
//!
//! Provides a connection pool, typed row structs, and repository functions
//! for every table in the rusty-automation schema.  No business logic lives here.

pub mod error;
pub mod pool;
pub mod repository;
pub mod models;

pub use pool::DbPool;
pub use error::DbError;
