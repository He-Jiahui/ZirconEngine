use zircon_runtime_interface::world_sync::{WorldHierarchyRow, WorldQueryResult};
use zircon_runtime_interface::{GatewaySessionIdentity, ZrRuntimeSessionHandle};

use super::PlayHierarchyProjection;

fn identity(gateway_generation: u64) -> GatewaySessionIdentity {
    GatewaySessionIdentity::new(3, ZrRuntimeSessionHandle::new(5), 7, None)
        .with_play_instance(Some(11))
        .with_gateway_generation(gateway_generation)
}

fn row(entity: u64, parent: Option<u64>, depth: u32, display_name: &str) -> WorldHierarchyRow {
    WorldHierarchyRow {
        entity,
        parent,
        depth,
        display_name: display_name.to_string(),
        kind: "Entity".to_string(),
        subtree_hash: entity.wrapping_mul(17),
        active_in_hierarchy: true,
        has_children: false,
    }
}

#[test]
fn first_identity_snapshot_requires_a_complete_reflow() {
    let mut projection = PlayHierarchyProjection::default();

    let fragment = projection
        .apply(
            identity(1),
            WorldQueryResult::HierarchyRows {
                generation: 4,
                rows: vec![row(1, None, 0, "Root")],
            },
            Some(1),
            8,
            [1],
            false,
        )
        .expect("first hierarchy response should be valid")
        .expect("first hierarchy response should publish a fragment");

    assert!(fragment.reflow_entries().is_some());
    assert_eq!(fragment.message().generation(), 4);
    assert!(fragment.message().requires_resync());
}

#[test]
fn world_replacement_clear_removes_the_generation_hint_and_rows() {
    let current_identity = identity(1);
    let mut projection = PlayHierarchyProjection::default();
    projection
        .apply(
            current_identity.clone(),
            WorldQueryResult::HierarchyRows {
                generation: 4,
                rows: vec![row(1, None, 0, "Root")],
            },
            Some(1),
            8,
            [1],
            false,
        )
        .unwrap();

    assert!(projection.clear());
    assert_eq!(projection.generation_hint(&current_identity), None);
    assert_eq!(projection.row(1), None);
    assert!(!projection.clear());
}

#[test]
fn same_topology_value_change_uses_a_sparse_patch() {
    let mut projection = PlayHierarchyProjection::default();
    projection
        .apply(
            identity(1),
            WorldQueryResult::HierarchyRows {
                generation: 4,
                rows: vec![row(1, None, 0, "Before")],
            },
            Some(1),
            8,
            [1],
            false,
        )
        .expect("base hierarchy should be valid");

    let fragment = projection
        .apply(
            identity(1),
            WorldQueryResult::HierarchyRows {
                generation: 5,
                rows: vec![row(1, None, 0, "After")],
            },
            Some(1),
            8,
            [1],
            false,
        )
        .expect("changed hierarchy should be valid")
        .expect("changed hierarchy should publish a fragment");

    assert_eq!(fragment.changed_rows().map(|rows| rows.len()), Some(1));
    assert!(fragment.reflow_entries().is_none());
    assert_eq!(fragment.message().previous_generation(), Some(4));
    assert_eq!(fragment.message().changed_anchors().len(), 1);
    let anchor = &fragment.message().changed_anchors()[0];
    assert_eq!(anchor.entity(), 1);
    assert_eq!(anchor.parent(), None);
    assert_eq!(anchor.depth(), 0);
    assert_eq!(anchor.subtree_hash(), 17);
}

#[test]
fn spawn_or_reparent_forces_a_complete_reflow() {
    let mut projection = PlayHierarchyProjection::default();
    projection
        .apply(
            identity(1),
            WorldQueryResult::HierarchyRows {
                generation: 4,
                rows: vec![row(1, None, 0, "Root")],
            },
            None,
            1,
            [],
            false,
        )
        .expect("base hierarchy should be valid");

    let fragment = projection
        .apply(
            identity(1),
            WorldQueryResult::HierarchyRows {
                generation: 5,
                rows: vec![row(1, None, 0, "Root"), row(2, Some(1), 1, "Child")],
            },
            None,
            1,
            [],
            false,
        )
        .expect("structural hierarchy should be valid")
        .expect("structural hierarchy should publish a fragment");

    assert_eq!(fragment.reflow_entries().map(|rows| rows.len()), Some(2));
    assert!(fragment.message().requires_resync());
}

#[test]
fn not_modified_can_still_advance_the_play_selection_overlay() {
    let mut projection = PlayHierarchyProjection::default();
    projection
        .apply(
            identity(1),
            WorldQueryResult::HierarchyRows {
                generation: 4,
                rows: vec![row(1, None, 0, "Root")],
            },
            None,
            2,
            [],
            false,
        )
        .expect("base hierarchy should be valid");

    let fragment = projection
        .apply(
            identity(1),
            WorldQueryResult::NotModified { generation: 4 },
            Some(1),
            3,
            [1],
            false,
        )
        .expect("selection-only update should be valid")
        .expect("selection-only update should publish a fragment");

    assert!(fragment.changed_rows().is_some_and(|rows| rows.is_empty()));
    assert_eq!(fragment.message().selection().previous_revision(), Some(2));
    assert_eq!(fragment.message().selection().added_entities(), &[1]);
}
