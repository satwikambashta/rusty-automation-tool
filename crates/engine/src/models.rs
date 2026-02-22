//! Core domain models for the workflow engine.
//!
//! These types are the source of truth for what a workflow looks like
//! in memory.  They can be serialised to/from the JSONB `definition`
//! column of the `workflows` table.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trigger
// ---------------------------------------------------------------------------

/// How a workflow is started.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Trigger {
    /// Triggered by an incoming HTTP request to `/webhook/{path}`.
    Webhook {
        /// URL path segment that identifies this workflow.
        path: String,
    },
    /// Triggered manually via the REST API.
    Manual,
    /// Triggered on a cron schedule.
    Cron {
        /// Standard cron expression (5 fields).
        expression: String,
    },
}

// ---------------------------------------------------------------------------
// NodeDefinition
// ---------------------------------------------------------------------------

/// A single step in the workflow graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDefinition {
    /// Unique identifier within this workflow (referenced by edges).
    pub id: String,
    /// Maps to a registered `ExecutableNode` implementation.
    pub node_type: String,
    /// Arbitrary configuration passed to the node at execution time.
    pub config: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Edge
// ---------------------------------------------------------------------------

/// Directed edge from one node to another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
}

// ---------------------------------------------------------------------------
// Workflow
// ---------------------------------------------------------------------------

/// A complete workflow definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub trigger: Trigger,
    pub nodes: Vec<NodeDefinition>,
    pub edges: Vec<Edge>,
    pub created_at: DateTime<Utc>,
}

impl Workflow {
    /// Convenience constructor for testing.
    pub fn new(
        name: impl Into<String>,
        trigger: Trigger,
        nodes: Vec<NodeDefinition>,
        edges: Vec<Edge>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            trigger,
            nodes,
            edges,
            created_at: Utc::now(),
        }
    }
}
