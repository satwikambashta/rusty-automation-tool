//! Row structs that map 1-to-1 onto database tables.
//!
//! These are *persistence* models — they carry no domain behaviour.
//! Domain types live in the `engine` crate.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// workflows
// ---------------------------------------------------------------------------

/// A persisted workflow definition row.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkflowRow {
    pub id: Uuid,
    pub name: String,
    /// Full JSON workflow definition (nodes, edges, trigger, …)
    pub definition: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// workflow_executions
// ---------------------------------------------------------------------------

/// Possible statuses for a workflow execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}

impl std::fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Succeeded => write!(f, "succeeded"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for ExecutionStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending"   => Ok(Self::Pending),
            "running"   => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed"    => Ok(Self::Failed),
            other       => Err(format!("unknown execution status: {other}")),
        }
    }
}

/// A persisted workflow execution row.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkflowExecutionRow {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// node_executions
// ---------------------------------------------------------------------------

/// A persisted node execution row.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NodeExecutionRow {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub node_id: String,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// secrets
// ---------------------------------------------------------------------------

/// A persisted secret row.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SecretRow {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub key: String,
    /// AES-256 encrypted value (base64-encoded ciphertext).
    pub encrypted_value: String,
}

// ---------------------------------------------------------------------------
// job_queue
// ---------------------------------------------------------------------------

/// Possible statuses for a queued job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    DeadLettered,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending      => write!(f, "pending"),
            Self::Processing   => write!(f, "processing"),
            Self::Completed    => write!(f, "completed"),
            Self::Failed       => write!(f, "failed"),
            Self::DeadLettered => write!(f, "dead_lettered"),
        }
    }
}

/// A job row fetched from the queue table.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct JobRow {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub workflow_id: Uuid,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
