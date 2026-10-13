use super::*;
use crate::scene::ecs::{ComponentId, QueryAccess};

fn access(writes: bool) -> SystemParamAccess {
    let mut query = QueryAccess::default();
    if writes {
        query.add_write(ComponentId::new(9)).unwrap();
    } else {
        query.add_filter_read(ComponentId::new(9));
    }
    let mut access = SystemParamAccess::default();
    access.add_query_access(&query).unwrap();
    access
}

#[test]
fn duplicate_ids_keep_all_conflicts_visible_in_both_diagnostic_queries() {
    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new("duplicate", SystemStage::Update, access(true)),
        ScheduleConflictNode::new("reader", SystemStage::Update, access(false)),
        ScheduleConflictNode::new(
            "duplicate",
            SystemStage::Update,
            SystemParamAccess::default(),
        ),
    ]);
    assert!(graph.has_conflicts());
    assert!(graph.systems_conflict("duplicate", "reader"));
    assert!(graph.systems_conflict("reader", "duplicate"));
    assert_eq!(graph.conflicts_for("duplicate").count(), 1);
    assert!(!graph.systems_conflict("missing", "reader"));
}

#[test]
fn duplicate_endpoint_edges_are_reported_once_and_batches_keep_identity_unique() {
    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new("duplicate", SystemStage::Update, access(true)),
        ScheduleConflictNode::new("duplicate", SystemStage::Update, access(true)),
    ]);
    assert!(graph.systems_conflict("duplicate", "duplicate"));
    assert_eq!(graph.conflicts_for("duplicate").count(), 1);
    let independent = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "duplicate",
            SystemStage::Update,
            SystemParamAccess::default(),
        ),
        ScheduleConflictNode::new(
            "duplicate",
            SystemStage::Update,
            SystemParamAccess::default(),
        ),
    ]);
    assert_eq!(independent.conservative_parallel_batches().len(), 2);
}
