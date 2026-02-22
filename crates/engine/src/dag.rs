//! DAG validation — run this before persisting or executing a workflow.
//!
//! Rules enforced:
//! 1. Node IDs must be unique within the workflow.
//! 2. Every edge must reference valid node IDs (both `from` and `to`).
//! 3. The directed graph must be acyclic (topological sort must succeed).
//!
//! Returns a topologically-sorted list of node IDs on success.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::{EngineError, models::Workflow};

/// Validate the workflow's DAG and return nodes in topological execution order.
///
/// # Errors
/// - [`EngineError::DuplicateNodeId`] if two nodes share an ID.
/// - [`EngineError::UnknownNodeReference`] if an edge references a missing node.
/// - [`EngineError::CycleDetected`] if the graph is not acyclic.
pub fn validate_dag(workflow: &Workflow) -> Result<Vec<String>, EngineError> {
    // -----------------------------------------------------------------------
    // 1. Ensure node IDs are unique
    // -----------------------------------------------------------------------
    let mut seen_ids: HashSet<&str> = HashSet::new();
    for node in &workflow.nodes {
        if !seen_ids.insert(node.id.as_str()) {
            return Err(EngineError::DuplicateNodeId(node.id.clone()));
        }
    }

    let node_set: HashSet<&str> = workflow.nodes.iter().map(|n| n.id.as_str()).collect();

    // -----------------------------------------------------------------------
    // 2. Validate edge endpoints
    // -----------------------------------------------------------------------
    for edge in &workflow.edges {
        if !node_set.contains(edge.from.as_str()) {
            return Err(EngineError::UnknownNodeReference {
                node_id: edge.from.clone(),
                side: "from",
            });
        }
        if !node_set.contains(edge.to.as_str()) {
            return Err(EngineError::UnknownNodeReference {
                node_id: edge.to.clone(),
                side: "to",
            });
        }
    }

    // -----------------------------------------------------------------------
    // 3. Topological sort (Kahn's algorithm)
    // -----------------------------------------------------------------------
    // Build adjacency list and in-degree map.
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut in_degree: HashMap<&str, usize> = HashMap::new();

    for node in &workflow.nodes {
        adjacency.entry(node.id.as_str()).or_default();
        in_degree.entry(node.id.as_str()).or_insert(0);
    }

    for edge in &workflow.edges {
        adjacency
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
        *in_degree.entry(edge.to.as_str()).or_insert(0) += 1;
    }

    // Seed the queue with nodes that have no incoming edges.
    let mut queue: VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut sorted: Vec<String> = Vec::with_capacity(workflow.nodes.len());

    while let Some(node_id) = queue.pop_front() {
        sorted.push(node_id.to_owned());

        if let Some(neighbours) = adjacency.get(node_id) {
            for &neighbour in neighbours {
                let deg = in_degree.entry(neighbour).or_insert(0);
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(neighbour);
                }
            }
        }
    }

    // If we didn't visit every node the graph contains a cycle.
    if sorted.len() != workflow.nodes.len() {
        return Err(EngineError::CycleDetected);
    }

    Ok(sorted)
}

// ============================================================
// Unit tests
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Edge, NodeDefinition, Trigger};
    use uuid::Uuid;
    use chrono::Utc;

    fn make_node(id: &str) -> NodeDefinition {
        NodeDefinition {
            id: id.to_string(),
            node_type: "mock".into(),
            config: serde_json::Value::Null,
        }
    }

    fn make_workflow(nodes: Vec<NodeDefinition>, edges: Vec<Edge>) -> Workflow {
        Workflow {
            id: Uuid::new_v4(),
            name: "test".into(),
            trigger: Trigger::Manual,
            nodes,
            edges,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn valid_linear_dag_returns_sorted_order() {
        // A → B → C
        let workflow = make_workflow(
            vec![make_node("a"), make_node("b"), make_node("c")],
            vec![
                Edge { from: "a".into(), to: "b".into() },
                Edge { from: "b".into(), to: "c".into() },
            ],
        );

        let sorted = validate_dag(&workflow).expect("should be valid");
        assert_eq!(sorted, vec!["a", "b", "c"]);
    }

    #[test]
    fn valid_diamond_dag() {
        //   A
        //  / \
        // B   C
        //  \ /
        //   D
        let workflow = make_workflow(
            vec![make_node("a"), make_node("b"), make_node("c"), make_node("d")],
            vec![
                Edge { from: "a".into(), to: "b".into() },
                Edge { from: "a".into(), to: "c".into() },
                Edge { from: "b".into(), to: "d".into() },
                Edge { from: "c".into(), to: "d".into() },
            ],
        );

        let sorted = validate_dag(&workflow).expect("should be valid");
        // 'a' must be first, 'd' must be last.
        assert_eq!(sorted.first().unwrap(), "a");
        assert_eq!(sorted.last().unwrap(), "d");
        assert_eq!(sorted.len(), 4);
    }

    #[test]
    fn duplicate_node_id_is_rejected() {
        let workflow = make_workflow(
            vec![make_node("a"), make_node("a")], // duplicate!
            vec![],
        );
        assert!(matches!(
            validate_dag(&workflow),
            Err(EngineError::DuplicateNodeId(id)) if id == "a"
        ));
    }

    #[test]
    fn edge_referencing_missing_node_is_rejected() {
        let workflow = make_workflow(
            vec![make_node("a")],
            vec![Edge { from: "a".into(), to: "ghost".into() }], // ghost doesn't exist
        );
        assert!(matches!(
            validate_dag(&workflow),
            Err(EngineError::UnknownNodeReference { node_id, .. }) if node_id == "ghost"
        ));
    }

    #[test]
    fn cycle_is_detected() {
        // A → B → C → A  (cycle!)
        let workflow = make_workflow(
            vec![make_node("a"), make_node("b"), make_node("c")],
            vec![
                Edge { from: "a".into(), to: "b".into() },
                Edge { from: "b".into(), to: "c".into() },
                Edge { from: "c".into(), to: "a".into() }, // back-edge
            ],
        );
        assert!(matches!(validate_dag(&workflow), Err(EngineError::CycleDetected)));
    }

    #[test]
    fn single_node_no_edges_is_valid() {
        let workflow = make_workflow(vec![make_node("solo")], vec![]);
        let sorted = validate_dag(&workflow).expect("single node should be valid");
        assert_eq!(sorted, vec!["solo"]);
    }
}
