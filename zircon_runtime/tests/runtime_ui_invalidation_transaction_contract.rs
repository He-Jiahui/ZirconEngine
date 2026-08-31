use zircon_runtime::ui::surface::{
    UiInvalidationApplyError, UiInvalidationGenerations, UiInvalidationReason,
    UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface,
    UiSurfaceInvalidationApplyError, UiSurfaceInvalidationState,
};
use zircon_runtime_interface::ui::{
    component::UiValue,
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{AxisConstraint, BoxConstraints, StretchMode, UiSize, UiSlot, UiSlotKind},
    tree::{UiDirtyFlags, UiTreeError, UiTreeNode},
};

#[test]
fn invalidation_reason_matrix_advances_every_downstream_domain_once() {
    let cases = [
        (
            UiInvalidationReason::Structure,
            generations(1, 1, 0, 1, 1, 1, 0),
        ),
        (
            UiInvalidationReason::Layout,
            generations(0, 1, 0, 1, 1, 0, 0),
        ),
        (UiInvalidationReason::Text, generations(0, 1, 1, 1, 1, 0, 0)),
        (
            UiInvalidationReason::HitTest,
            generations(0, 0, 0, 1, 0, 0, 0),
        ),
        (
            UiInvalidationReason::Render,
            generations(0, 0, 0, 0, 1, 0, 0),
        ),
        (
            UiInvalidationReason::Interaction,
            generations(0, 0, 0, 1, 1, 1, 0),
        ),
        (
            UiInvalidationReason::Resource,
            generations(0, 1, 0, 1, 1, 0, 1),
        ),
    ];

    for (reason, expected) in cases {
        let mut state = UiSurfaceInvalidationState::default();
        state.record_reason(UiNodeId::new(7), reason);
        let commit = state
            .commit_pending()
            .unwrap()
            .expect("reason should publish one invalidation generation");

        assert_eq!(commit.generations, expected, "reason: {reason:?}");
        assert_eq!(commit.changed_nodes.len(), 1, "reason: {reason:?}");
    }
}

#[test]
fn dirty_flag_matrix_includes_implicit_layout_and_input_consumers() {
    let cases = [
        (
            UiDirtyFlags {
                visible_range: true,
                ..Default::default()
            },
            generations(0, 1, 0, 1, 1, 0, 0),
        ),
        (
            UiDirtyFlags {
                input: true,
                ..Default::default()
            },
            generations(0, 0, 0, 1, 0, 1, 0),
        ),
    ];

    for (dirty, expected) in cases {
        let mut state = UiSurfaceInvalidationState::default();
        state.record_dirty(UiNodeId::new(8), dirty);
        let commit = state
            .commit_pending()
            .unwrap()
            .expect("dirty flags should publish one invalidation generation");

        assert_eq!(commit.generations, expected, "dirty: {dirty:?}");
    }
}

#[test]
fn invalidation_transaction_merges_nodes_and_advances_each_domain_once() {
    let mut state = UiSurfaceInvalidationState::default();
    let node_id = UiNodeId::new(7);

    state.record_reason(node_id, UiInvalidationReason::Layout);
    state.record_reason(node_id, UiInvalidationReason::Render);
    state.record_dirty(
        node_id,
        UiDirtyFlags {
            text: true,
            render: true,
            ..Default::default()
        },
    );

    let commit = state
        .commit_pending()
        .expect("current pending transaction should commit")
        .expect("non-empty transaction should publish");
    assert_eq!(commit.base_generation, 0);
    assert_eq!(commit.generation, 1);
    assert_eq!(commit.changed_nodes.len(), 1);
    assert_eq!(commit.changed_nodes[0].node_id, node_id);
    assert_eq!(
        commit.changed_nodes[0].dirty,
        UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            text: true,
            ..Default::default()
        }
    );
    assert_eq!(commit.generations, generations(0, 1, 1, 1, 1, 0, 0));
}

#[test]
fn empty_and_stale_transactions_do_not_publish_a_generation() {
    let mut state = UiSurfaceInvalidationState::default();

    assert_eq!(state.commit_pending().unwrap(), None);
    assert_eq!(state.generations().generation, 0);

    let mut stale = state.begin_transaction();
    let mut winner = state.begin_transaction();
    winner.record_reason(UiNodeId::new(1), UiInvalidationReason::Interaction);
    state
        .apply_transaction(winner)
        .expect("current transaction should apply")
        .expect("winner should publish");

    stale.record_reason(UiNodeId::new(2), UiInvalidationReason::Render);
    assert_eq!(
        state.apply_transaction(stale),
        Err(UiInvalidationApplyError::StaleGeneration {
            expected: 0,
            actual: 1,
        })
    );
    assert_eq!(state.generations().generation, 1);
}

#[test]
fn surface_transaction_marks_real_nodes_and_publishes_only_after_rebuild() {
    let mut surface = test_surface();
    let baseline = surface.invalidation_generations();
    let mut transaction = surface.begin_invalidation_transaction();
    transaction.record_reason(UiNodeId::new(2), UiInvalidationReason::Render);

    surface
        .apply_invalidation_transaction(transaction)
        .expect("current transaction should atomically mark the retained tree");
    assert_eq!(
        surface.invalidation_generations().generation,
        baseline.generation
    );
    assert!(surface.tree.node(UiNodeId::new(2)).unwrap().dirty.render);
    assert_eq!(surface.pending_invalidation_changed_node_count(), 1);

    surface.rebuild_dirty(root_size()).unwrap();
    assert_eq!(
        surface.invalidation_generations().generation,
        baseline.generation + 1
    );
    assert_eq!(surface.pending_invalidation_changed_node_count(), 0);

    let mut stale =
        zircon_runtime::ui::surface::UiInvalidationTransaction::new(baseline.generation);
    stale.record_reason(UiNodeId::new(1), UiInvalidationReason::Render);
    assert_eq!(
        surface.apply_invalidation_transaction(stale),
        Err(UiSurfaceInvalidationApplyError::InvalidTransaction(
            UiInvalidationApplyError::StaleGeneration {
                expected: baseline.generation,
                actual: baseline.generation + 1,
            }
        ))
    );
    assert_eq!(surface.dirty_flags(), UiDirtyFlags::default());
}

#[test]
fn surface_transaction_rejects_missing_nodes_without_partial_marks() {
    let mut surface = test_surface();
    let mut transaction = surface.begin_invalidation_transaction();
    transaction.record_reason(UiNodeId::new(2), UiInvalidationReason::Render);
    transaction.record_reason(UiNodeId::new(999), UiInvalidationReason::Render);

    assert_eq!(
        surface.apply_invalidation_transaction(transaction),
        Err(UiSurfaceInvalidationApplyError::Tree(
            UiTreeError::MissingNode(UiNodeId::new(999))
        ))
    );
    assert_eq!(surface.dirty_flags(), UiDirtyFlags::default());
    assert_eq!(surface.pending_invalidation_changed_node_count(), 0);
}

#[test]
fn force_rebuild_does_not_publish_layout_generation_before_layout_finishes() {
    let mut surface = test_surface();
    let baseline = surface.invalidation_generations();
    surface
        .invalidate_node(UiNodeId::new(2), UiInvalidationReason::Layout)
        .unwrap();

    surface.rebuild();
    assert_eq!(
        surface.invalidation_generations().generation,
        baseline.generation
    );
    assert!(surface.dirty_flags().layout);
    assert_eq!(surface.pending_invalidation_changed_node_count(), 1);

    surface.rebuild_dirty(root_size()).unwrap();
    assert_eq!(
        surface.invalidation_generations().generation,
        baseline.generation + 1
    );
    assert_eq!(surface.dirty_flags(), UiDirtyFlags::default());

    surface.rebuild_dirty(root_size()).unwrap();
    assert_eq!(
        surface.invalidation_generations().generation,
        baseline.generation + 1
    );
}

#[test]
fn serialized_surface_preserves_pending_invalidation_until_rebuild() {
    let mut surface = test_surface();
    let baseline = surface.invalidation_generations();
    surface
        .invalidate_node(UiNodeId::new(2), UiInvalidationReason::Text)
        .unwrap();

    let serialized = serde_json::to_string(&surface).unwrap();
    let mut restored: UiSurface = serde_json::from_str(&serialized).unwrap();

    assert_eq!(restored.invalidation_generations(), baseline);
    assert_eq!(restored.pending_invalidation_changed_node_count(), 1);
    assert!(restored.dirty_flags().text);

    restored.rebuild_dirty(root_size()).unwrap();
    let committed = restored.invalidation_generations();
    assert_eq!(committed.generation, baseline.generation + 1);
    assert_eq!(committed.layout, baseline.layout + 1);
    assert_eq!(committed.text, baseline.text + 1);
    assert_eq!(committed.hit_test, baseline.hit_test + 1);
    assert_eq!(committed.render, baseline.render + 1);
}

#[test]
fn rebuild_reconciles_direct_descendant_and_state_flag_changes() {
    let mut surface = test_surface();
    let baseline = surface.invalidation_generations();
    surface
        .tree
        .node_mut(UiNodeId::new(1))
        .unwrap()
        .state_flags
        .dirty = true;
    surface.tree.node_mut(UiNodeId::new(2)).unwrap().dirty.text = true;

    surface.rebuild_dirty(root_size()).unwrap();

    let commit = surface
        .last_invalidation_commit()
        .expect("tree dirty scan should reconcile every affected node");
    assert_eq!(commit.changed_nodes.len(), 2);
    assert!(commit
        .changed_nodes
        .iter()
        .any(|change| change.node_id == UiNodeId::new(1) && change.dirty.input));
    assert!(commit
        .changed_nodes
        .iter()
        .any(|change| change.node_id == UiNodeId::new(2) && change.dirty.text));
    assert_eq!(commit.generations.text, baseline.text + 1);
    assert_eq!(commit.generations.interaction, baseline.interaction + 1);
}

#[test]
fn node_pool_and_slot_mutations_publish_explicit_structure_generations() {
    let mut slot_surface = test_surface();
    let slot_baseline = slot_surface.invalidation_generations();
    slot_surface.tree.push_layout_slot(UiSlot::new(
        UiNodeId::new(1),
        UiNodeId::new(2),
        UiSlotKind::Overlay,
    ));
    assert!(slot_surface
        .set_overlay_slot_z_order(UiNodeId::new(1), UiNodeId::new(2), 4)
        .unwrap());
    slot_surface.rebuild_dirty(root_size()).unwrap();
    assert_eq!(
        slot_surface.invalidation_generations().structure,
        slot_baseline.structure + 1
    );

    let mut pool_surface = test_surface();
    let pool_baseline = pool_surface.invalidation_generations();
    pool_surface
        .detach_subtree_to_pool(UiNodeId::new(2))
        .unwrap();
    pool_surface.rebuild_dirty(root_size()).unwrap();
    let commit = pool_surface
        .last_invalidation_commit()
        .expect("detach should publish structure invalidation");
    assert_eq!(commit.generations.structure, pool_baseline.structure + 1);
    assert!(commit
        .changed_nodes
        .iter()
        .any(|change| change.node_id == UiNodeId::new(2)));
}

#[test]
fn public_tree_insert_child_cannot_bypass_structure_invalidation() {
    let mut surface = test_surface();
    let baseline = surface.invalidation_generations();
    let inserted_id = UiNodeId::new(3);

    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(inserted_id, UiNodePath::new("root/inserted")).with_constraints(
                BoxConstraints {
                    width: fixed_constraint(24.0),
                    height: fixed_constraint(12.0),
                },
            ),
        )
        .unwrap();

    assert!(surface.dirty_flags().layout);
    surface.rebuild_dirty(root_size()).unwrap();

    let commit = surface
        .last_invalidation_commit()
        .expect("public topology mutation should publish invalidation");
    assert_eq!(commit.generation, baseline.generation + 1);
    assert_eq!(commit.generations.structure, baseline.structure + 1);
    assert!(commit
        .changed_nodes
        .iter()
        .any(|change| change.node_id == inserted_id));
    assert!(commit
        .changed_nodes
        .iter()
        .any(|change| change.node_id == UiNodeId::new(1)));
}

#[test]
fn changed_node_set_cost_tracks_one_change_on_real_surface_sizes() {
    for surface_node_count in [1_u64, 100, 10_000] {
        let mut surface = surface_with_roots(surface_node_count);
        assert_eq!(surface.tree.nodes.len(), surface_node_count as usize);
        surface.rebuild();

        let changed_node_id = UiNodeId::new(surface_node_count);
        surface
            .invalidate_node(changed_node_id, UiInvalidationReason::Render)
            .unwrap();
        surface.rebuild_dirty(root_size()).unwrap();

        let commit = surface
            .last_invalidation_commit()
            .expect("single-node change should publish");
        assert_eq!(commit.changed_nodes.len(), 1);
        assert_eq!(commit.changed_nodes[0].node_id, changed_node_id);
    }
}

#[test]
fn surface_merges_reentrant_dirty_marks_into_one_committed_node() {
    let mut surface = test_surface();
    let baseline = surface.invalidation_generations();
    let node_id = UiNodeId::new(2);

    surface
        .mark_node_dirty(
            node_id,
            UiDirtyFlags {
                render: true,
                ..Default::default()
            },
        )
        .unwrap();
    surface
        .mark_node_dirty(
            node_id,
            UiDirtyFlags {
                hit_test: true,
                input: true,
                ..Default::default()
            },
        )
        .unwrap();

    assert_eq!(surface.pending_invalidation_changed_node_count(), 1);
    surface.rebuild_dirty(root_size()).unwrap();

    let commit = surface
        .last_invalidation_commit()
        .expect("dirty rebuild should publish invalidation");
    assert_eq!(commit.generation, baseline.generation + 1);
    assert_eq!(commit.changed_nodes.len(), 1);
    assert!(commit.changed_nodes[0].dirty.hit_test);
    assert!(commit.changed_nodes[0].dirty.render);
    assert!(commit.changed_nodes[0].dirty.input);
}

#[test]
fn unchanged_property_and_stable_rebuild_do_not_advance_generation() {
    let mut surface = test_surface();
    let baseline = surface.invalidation_generations();
    let request =
        || UiPropertyMutationRequest::new(UiNodeId::new(2), "pressed", UiValue::Bool(true));

    assert_eq!(
        surface.mutate_property(request()).unwrap().status,
        UiPropertyMutationStatus::Accepted
    );
    surface.rebuild_dirty(root_size()).unwrap();
    assert_eq!(
        surface.invalidation_generations().generation,
        baseline.generation + 1
    );

    assert_eq!(
        surface.mutate_property(request()).unwrap().status,
        UiPropertyMutationStatus::Unchanged
    );
    surface.rebuild_dirty(root_size()).unwrap();
    assert_eq!(
        surface.invalidation_generations().generation,
        baseline.generation + 1
    );
    assert_eq!(surface.pending_invalidation_changed_node_count(), 0);
}

fn generations(
    structure: u64,
    layout: u64,
    text: u64,
    hit_test: u64,
    render: u64,
    interaction: u64,
    resource: u64,
) -> UiInvalidationGenerations {
    UiInvalidationGenerations {
        generation: 1,
        structure,
        layout,
        text,
        hit_test,
        render,
        interaction,
        resource,
    }
}

fn surface_with_roots(node_count: u64) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(format!(
        "runtime.ui.invalidation.scale.{node_count}"
    )));
    for raw_id in 1..=node_count {
        let node_id = UiNodeId::new(raw_id);
        let mut node = UiTreeNode::new(node_id, UiNodePath::new(format!("root/{raw_id}")));
        node.paint_order = raw_id - 1;
        surface.tree.roots.push(node_id);
        surface.tree.nodes.insert(node_id, node);
    }
    surface
}

fn test_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.invalidation"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_constraints(
            BoxConstraints {
                width: fixed_constraint(120.0),
                height: fixed_constraint(60.0),
            },
        ),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/button")).with_constraints(
                BoxConstraints {
                    width: fixed_constraint(80.0),
                    height: fixed_constraint(24.0),
                },
            ),
        )
        .unwrap();
    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();
    surface
}

fn fixed_constraint(value: f32) -> AxisConstraint {
    AxisConstraint {
        min: value,
        max: value,
        preferred: value,
        stretch_mode: StretchMode::Fixed,
        ..Default::default()
    }
}

fn root_size() -> UiSize {
    UiSize {
        width: 120.0,
        height: 60.0,
    }
}
