use zircon_runtime_interface::world_sync::{WorldHierarchyRow, WorldQueryResult};
use zircon_runtime_interface::{GatewaySessionIdentity, ZrRuntimeSessionHandle};

use crate::ui::workbench::snapshot::SceneInspectionHierarchyFragment;

use super::{PlayHierarchyProjection, PlayHierarchyProjectionError};

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

pub(super) fn apply_same_topology_change(
    row_count: usize,
    changed_count: usize,
) -> (
    PlayHierarchyProjection,
    Option<SceneInspectionHierarchyFragment>,
) {
    assert!(changed_count <= row_count);
    let mut projection = PlayHierarchyProjection::default();
    let mut rows = (1..=row_count as u64)
        .map(|entity| row(entity, None, 0, "Before"))
        .collect::<Vec<_>>();
    projection
        .apply(
            identity(1),
            WorldQueryResult::HierarchyRows {
                generation: 4,
                rows: rows.clone(),
            },
            Some(1),
            2,
            [1],
            false,
        )
        .unwrap()
        .expect("a new projection must reflow its base");
    for row in rows.iter_mut().take(changed_count) {
        row.display_name = "After".to_string();
    }
    let fragment = projection
        .apply(
            identity(1),
            WorldQueryResult::HierarchyRows {
                generation: if changed_count == 0 { 4 } else { 5 },
                rows,
            },
            Some(1),
            2,
            [1],
            false,
        )
        .unwrap();
    (projection, fragment)
}

#[test]
fn same_topology_changed_row_capacity_matches_zero_sparse_and_dense_changes() {
    const ROW_COUNT: usize = 4096;
    for changed_count in [0, 1, ROW_COUNT] {
        let (mut projection, fragment) = apply_same_topology_change(ROW_COUNT, changed_count);
        let capacity = projection
            .changed_row_capacity
            .as_ref()
            .expect("the actual changed-row collector must have executed");
        assert_eq!(capacity.initial_capacity, changed_count);
        assert_eq!(capacity.final_capacity, changed_count);
        assert_eq!(capacity.growth_events, 0);
        if changed_count == 0 {
            assert!(fragment.is_none());
            let advanced = projection
                .apply(
                    identity(1),
                    WorldQueryResult::HierarchyRows {
                        generation: 5,
                        rows: (1..=ROW_COUNT as u64)
                            .map(|entity| row(entity, None, 0, "Before"))
                            .collect(),
                    },
                    Some(1),
                    2,
                    [1],
                    false,
                )
                .unwrap()
                .expect("an advanced generation must publish even an empty delta");
            assert!(advanced.changed_rows().unwrap().is_empty());
            assert_eq!(advanced.message().previous_generation(), Some(4));
            assert_eq!(advanced.message().generation(), 5);
            let capacity = projection.changed_row_capacity.as_ref().unwrap();
            assert_eq!(capacity.initial_capacity, 0);
            assert_eq!(capacity.final_capacity, 0);
            assert_eq!(capacity.growth_events, 0);
            continue;
        }
        let fragment = fragment.expect("changed rows must publish a patch");
        assert!(fragment.reflow_entries().is_none());
        let changed_rows = fragment.changed_rows().unwrap();
        assert_eq!(changed_rows.len(), changed_count);
        assert_eq!(fragment.message().changed_anchors().len(), changed_count);
        assert_eq!(
            changed_rows
                .iter()
                .map(|row| row.entity)
                .collect::<Vec<_>>(),
            (1..=changed_count as u64).collect::<Vec<_>>()
        );
    }
}

#[test]
fn not_modified_force_reflow_updates_the_selection_delta_base() {
    let mut projection = PlayHierarchyProjection::default();
    projection
        .apply(
            identity(1),
            WorldQueryResult::HierarchyRows {
                generation: 4,
                rows: vec![row(1, None, 0, "First"), row(2, None, 0, "Second")],
            },
            Some(1),
            2,
            [1],
            false,
        )
        .unwrap();

    let forced = projection
        .apply(
            identity(1),
            WorldQueryResult::NotModified { generation: 4 },
            Some(2),
            3,
            [2],
            true,
        )
        .unwrap()
        .expect("forced reflow must publish the new selection");
    assert!(forced.message().requires_resync());
    assert_eq!(forced.message().selection().revision(), 3);
    let entries = forced.reflow_entries().unwrap();
    assert_eq!(
        entries
            .selected_entities()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![2]
    );
    assert!(projection
        .apply(
            identity(1),
            WorldQueryResult::NotModified { generation: 4 },
            Some(2),
            3,
            [2],
            false,
        )
        .unwrap()
        .is_none());

    let delta = projection
        .apply(
            identity(1),
            WorldQueryResult::NotModified { generation: 4 },
            Some(1),
            4,
            [1],
            false,
        )
        .unwrap()
        .expect("the next selection revision must publish a delta");
    assert!(delta.reflow_entries().is_none());
    let selection = delta.message().selection();
    assert_eq!(selection.previous_revision(), Some(3));
    assert_eq!(selection.revision(), 4);
    assert_eq!(selection.added_entities(), &[1]);
    assert_eq!(selection.removed_entities(), &[2]);
}

#[test]
fn same_generation_mutation_rejects_before_reflow_or_empty_delta() {
    for force_reflow in [false, true] {
        let mut projection = PlayHierarchyProjection::default();
        projection
            .apply(
                identity(1),
                WorldQueryResult::HierarchyRows {
                    generation: 4,
                    rows: vec![row(1, None, 0, "Before")],
                },
                Some(1),
                2,
                [1],
                false,
            )
            .unwrap();
        assert!(matches!(
            projection.apply(
                identity(1),
                WorldQueryResult::HierarchyRows {
                    generation: 4,
                    rows: vec![row(1, None, 0, "Changed without advancing")],
                },
                Some(1),
                2,
                [1],
                force_reflow,
            ),
            Err(PlayHierarchyProjectionError::SameGenerationMutation { generation: 4 })
        ));
        assert_eq!(projection.row(1).unwrap().display_name, "Before");
        let unchanged = projection
            .apply(
                identity(1),
                WorldQueryResult::HierarchyRows {
                    generation: 4,
                    rows: vec![row(1, None, 0, "Before")],
                },
                Some(1),
                2,
                [1],
                force_reflow,
            )
            .unwrap();
        if force_reflow {
            assert!(unchanged.unwrap().message().requires_resync());
        } else {
            assert!(unchanged.is_none());
        }
    }
}
