//! `MockNode` — a test double for `ExecutableNode`.
//!
//! Useful in unit and integration tests where a real node implementation is
//! either unavailable or irrelevant.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

use crate::{ExecutableNode, NodeError, traits::ExecutionContext};

/// Behaviour injected into `MockNode` at construction time.
pub enum MockBehaviour {
    /// Return a specific JSON value.
    ReturnValue(Value),
    /// Fail with a `Retryable` error.
    FailRetryable(String),
    /// Fail with a `Fatal` error.
    FailFatal(String),
}

/// A mock node that records every call it receives and returns a
/// programmer-specified result.
pub struct MockNode {
    /// Label used in test assertions.
    pub name: String,
    /// What the node will do when `execute` is called.
    pub behaviour: MockBehaviour,
    /// All inputs seen by this node (in call order).
    pub calls: Arc<Mutex<Vec<Value>>>,
}

impl MockNode {
    /// Create a mock that always succeeds with the given value.
    pub fn returning(name: impl Into<String>, value: Value) -> Self {
        Self {
            name: name.into(),
            behaviour: MockBehaviour::ReturnValue(value),
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a mock that always fails with a `Fatal` error.
    pub fn failing_fatal(name: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            behaviour: MockBehaviour::FailFatal(msg.into()),
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a mock that always fails with a `Retryable` error.
    pub fn failing_retryable(name: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            behaviour: MockBehaviour::FailRetryable(msg.into()),
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Number of times this node has been executed.
    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }
}

#[async_trait]
impl ExecutableNode for MockNode {
    async fn execute(&self, input: Value, _ctx: &ExecutionContext) -> Result<Value, NodeError> {
        self.calls.lock().unwrap().push(input.clone());

        match &self.behaviour {
            MockBehaviour::ReturnValue(v) => {
                // Merge the incoming input with the node's own output field so
                // tests can trace the data flowing through the pipeline.
                let mut out = json!({ "node": self.name });
                if let (Some(out_obj), Some(v_obj)) = (out.as_object_mut(), v.as_object()) {
                    for (k, val) in v_obj {
                        out_obj.insert(k.clone(), val.clone());
                    }
                }
                Ok(out)
            }
            MockBehaviour::FailRetryable(msg) => Err(NodeError::Retryable(msg.clone())),
            MockBehaviour::FailFatal(msg)     => Err(NodeError::Fatal(msg.clone())),
        }
    }
}
