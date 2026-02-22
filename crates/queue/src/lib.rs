//! `queue` crate — queue worker runtime (not yet implemented).
//!
//! Phase 1: workers poll the `job_queue` Postgres table.
//! Phase 2: swap in a Redis-backed queue with configurable concurrency.
