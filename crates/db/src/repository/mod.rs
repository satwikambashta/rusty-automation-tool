//! Repository functions — one function per database operation.
//!
//! Every function takes a `&DbPool` and returns a `Result<T, DbError>`.
//! No business logic, no domain types — pure SQL.

pub mod workflows;
pub mod executions;
pub mod jobs;
