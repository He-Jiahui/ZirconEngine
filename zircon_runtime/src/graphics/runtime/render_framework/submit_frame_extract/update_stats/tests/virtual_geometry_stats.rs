use std::collections::HashSet;

use super::{execution_state_for_page, RenderVirtualGeometryExecutionState};

#[test]
fn execution_state_uses_constant_time_page_membership() {
    let resident = HashSet::from([7]);
    let requested = HashSet::from([9]);

    assert_eq!(
        execution_state_for_page(7, &resident, &requested),
        RenderVirtualGeometryExecutionState::Resident
    );
    assert_eq!(
        execution_state_for_page(9, &resident, &requested),
        RenderVirtualGeometryExecutionState::PendingUpload
    );
    assert_eq!(
        execution_state_for_page(11, &resident, &requested),
        RenderVirtualGeometryExecutionState::Missing
    );
}

#[test]
fn traversal_stats_index_hierarchy_nodes_once() {
    let source = include_str!("../virtual_geometry_stats.rs");
    let linear_find = concat!(".find(", "|node| node.node_id == node_id)");

    assert!(source.contains("hierarchy_nodes_by_id"));
    assert!(source.contains("entry(node.node_id).or_insert(node)"));
    assert!(!source.contains(linear_find));
}
