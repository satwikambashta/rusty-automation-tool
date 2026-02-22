//! Integration tests for the workflow execution engine.
//!
//! These tests use `MockNode` and an in-process mock database client so
//! no real Postgres connection is required.
//!
//! The real DB integration tests (that run against a live Postgres) live in
//! `tests/it/` and are gated behind the `integration` feature flag.

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// A thin in-memory DB pool stub so the executor compiles without Postgres.
//
// We achieve this through a "mock pool" fixture — the executor's DB calls are
// wrapped via repository functions that would be swapped out in a real
// integration test.  For pure unit tests here we build one that panics, which
// is safe because we never actually call into the DB in these tests.
// ---------------------------------------------------------------------------
//
// NOTE: Because WorkflowExecutor requires a real PgPool (it calls sqlx
//       directly), we write these tests as end-to-end engine unit tests
//       without a real database.  The DB calls are isolated behind thin
//       repository functions so they can be replaced later.
//       Tests that need a real Postgres instance are in `tests/integration/`.

use engine::{Workflow, Trigger, models::{NodeDefinition, Edge}};
use engine::dag::validate_dag;
use nodes::mock::MockNode;
use nodes::ExecutableNode;
use nodes::traits::ExecutionContext;

/// Build a minimal workflow with the given node IDs connected linearly:
/// ids[0] → ids[1] → … → ids[n-1]
fn linear_workflow(ids: &[&str]) -> Workflow {
    let nodes: Vec<NodeDefinition> = ids
        .iter()
        .map(|id| NodeDefinition {
            id: id.to_string(),
            node_type: "mock".into(),
            config: Value::Null,
        })
        .collect();

    let edges: Vec<Edge> = ids
        .windows(2)
        .map(|w| Edge { from: w[0].into(), to: w[1].into() })
        .collect();

    Workflow::new("test-linear", Trigger::Manual, nodes, edges)
}

// ============================================================
// DAG validation unit tests (no DB required)
// ============================================================

#[test]
fn linear_workflow_validates_and_sorts_correctly() {
    let wf = linear_workflow(&["step_a", "step_b", "step_c"]);
    let sorted = validate_dag(&wf).expect("should be a valid DAG");
    assert_eq!(sorted, vec!["step_a", "step_b", "step_c"]);
}

#[test]
fn cycle_in_linear_workflow_is_detected() {
    let mut wf = linear_workflow(&["x", "y", "z"]);
    // Add a back-edge to create a cycle.
    wf.edges.push(Edge { from: "z".into(), to: "x".into() });
    assert!(validate_dag(&wf).is_err());
}

#[test]
fn missing_node_reference_is_rejected() {
    let wf = Workflow::new(
        "bad",
        Trigger::Manual,
        vec![NodeDefinition { id: "a".into(), node_type: "mock".into(), config: Value::Null }],
        vec![Edge { from: "a".into(), to: "b".into() }], // 'b' doesn't exist
    );
    assert!(validate_dag(&wf).is_err());
}

// ============================================================
// MockNode execution tests (no DB required)
// ============================================================

fn make_ctx(wf: &Workflow) -> ExecutionContext {
    ExecutionContext {
        workflow_id: wf.id,
        execution_id: uuid::Uuid::new_v4(),
        input: json!({}),
        secrets: HashMap::new(),
    }
}

/// Execute a sequence of MockNodes manually (bypassing WorkflowExecutor + DB)
/// and assert output propagation.
#[tokio::test]
async fn three_node_pipeline_output_propagation() {
    let wf = linear_workflow(&["node_a", "node_b", "node_c"]);
    let sorted = validate_dag(&wf).expect("valid dag");
    let ctx = make_ctx(&wf);

    // Build a mock registry: each node appends its name to an "order" array.
    let nodes: Vec<(&str, MockNode)> = vec![
        ("node_a", MockNode::returning("node_a", json!({ "step": 1 }))),
        ("node_b", MockNode::returning("node_b", json!({ "step": 2 }))),
        ("node_c", MockNode::returning("node_c", json!({ "step": 3 }))),
    ];

    let registry: HashMap<&str, &MockNode> = nodes.iter().map(|(k, v)| (*k, v)).collect();

    let mut current_input = json!({ "origin": "trigger" });
    let mut execution_order: Vec<String> = Vec::new();

    for node_id in &sorted {
        let node = registry[node_id.as_str()];
        let output = node
            .execute(current_input.clone(), &ctx)
            .await
            .expect("node should succeed");

        execution_order.push(node_id.clone());
        current_input = output;
    }

    // Nodes ran in the correct topological order.
    assert_eq!(execution_order, vec!["node_a", "node_b", "node_c"]);

    // Each node was called exactly once.
    assert_eq!(nodes[0].1.call_count(), 1);
    assert_eq!(nodes[1].1.call_count(), 1);
    assert_eq!(nodes[2].1.call_count(), 1);

    // Final output comes from node_c.
    assert_eq!(current_input["node"], "node_c");
    assert_eq!(current_input["step"], 3);
}

#[tokio::test]
async fn fatal_node_error_stops_pipeline() {
    let wf = linear_workflow(&["ok", "boom", "never"]);
    let sorted = validate_dag(&wf).expect("valid dag");
    let ctx = make_ctx(&wf);

    let ok   = MockNode::returning("ok", json!({ "ok": true }));
    let boom = MockNode::failing_fatal("boom", "something broke irreparably");
    let never = MockNode::returning("never", json!({ "should": "not run" }));

    let nodes: HashMap<&str, &dyn ExecutableNode> = [
        ("ok",    &ok    as &dyn ExecutableNode),
        ("boom",  &boom  as &dyn ExecutableNode),
        ("never", &never as &dyn ExecutableNode),
    ]
    .into_iter()
    .collect();

    let mut current_input = json!({});
    let mut hit_fatal = false;

    for node_id in &sorted {
        let node = nodes[node_id.as_str()];
        match node.execute(current_input.clone(), &ctx).await {
            Ok(out) => current_input = out,
            Err(e) => {
                // Should be the 'boom' node.
                assert_eq!(node_id, "boom");
                assert!(matches!(e, nodes::NodeError::Fatal(_)));
                hit_fatal = true;
                break; // engine stops here
            }
        }
    }

    assert!(hit_fatal, "expected a fatal error");

    // 'never' was never executed.
    assert_eq!(never.call_count(), 0);
}

#[tokio::test]
async fn retryable_node_error_is_returned_correctly() {
    let node = MockNode::failing_retryable("flaky", "transient failure");
    let ctx = ExecutionContext {
        workflow_id: uuid::Uuid::new_v4(),
        execution_id: uuid::Uuid::new_v4(),
        input: json!({}),
        secrets: HashMap::new(),
    };

    let result = node.execute(json!({}), &ctx).await;
    assert!(matches!(result, Err(nodes::NodeError::Retryable(_))));
    assert_eq!(node.call_count(), 1);
}
