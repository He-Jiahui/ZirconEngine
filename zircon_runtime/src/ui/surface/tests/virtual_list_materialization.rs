use super::{UiVirtualListItemKey, UiVirtualListMaterializationError};
use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    dispatch::UiPointerId,
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{UiContainerKind, UiScrollState, UiScrollableBoxConfig, UiVirtualListConfig},
    tree::UiTreeNode,
};

#[test]
fn rejects_a_non_virtualized_owner() {
    let mut surface = surface_with_owner(false);
    let mut changes = Vec::new();

    let error = surface
        .reconcile_virtual_list_materialization(owner_id(), 100, &mut changes)
        .unwrap_err();

    assert_eq!(
        error,
        UiVirtualListMaterializationError::NotVirtualizedOwner {
            owner_id: owner_id()
        }
    );
    assert!(changes.is_empty());
}

#[test]
fn one_row_scroll_rebinds_only_one_surface_owned_slot() {
    let mut surface = surface_with_owner(true);
    surface.tree.node_mut(owner_id()).unwrap().scroll_state = Some(UiScrollState {
        offset: 96.0,
        ..scroll_state()
    });
    let mut changes = Vec::new();
    let first = surface
        .reconcile_virtual_list_materialization(owner_id(), 100_000, &mut changes)
        .unwrap();
    let first_generation = first.generation;
    surface.tree.node_mut(owner_id()).unwrap().scroll_state = Some(UiScrollState {
        offset: 120.0,
        ..scroll_state()
    });

    let second = surface
        .reconcile_virtual_list_materialization(owner_id(), 100_000, &mut changes)
        .unwrap();

    assert_eq!(first.slot_capacity, 41);
    assert_eq!(second.slot_capacity, 41);
    assert_eq!(second.changed_slot_count, 1);
    assert_eq!(second.generation, first_generation + 1);
}

#[test]
fn identical_request_preserves_surface_owned_generation() {
    let mut surface = surface_with_owner(true);
    let mut changes = Vec::new();
    let first = surface
        .reconcile_virtual_list_materialization(owner_id(), 100_000, &mut changes)
        .unwrap();

    let second = surface
        .reconcile_virtual_list_materialization(owner_id(), 100_000, &mut changes)
        .unwrap();

    assert_eq!(second.generation, first.generation);
    assert_eq!(second.changed_slot_count, 0);
    assert!(changes.is_empty());
}

#[test]
fn warm_reconciliation_reuses_candidate_assignment_buffers() {
    let mut surface = surface_with_owner(true);
    let mut changes = Vec::new();
    for offset in [0.0, 24.0, 48.0] {
        surface.tree.node_mut(owner_id()).unwrap().scroll_state = Some(UiScrollState {
            offset,
            ..scroll_state()
        });
        surface
            .reconcile_virtual_list_materialization(owner_id(), 100_000, &mut changes)
            .unwrap();
    }

    let state = surface
        .virtual_list_materialization
        .owners
        .get(&owner_id())
        .expect("reconciliation creates owner state");
    let key_capacity = state.candidate_item_keys.capacity();
    let assignment_capacity = state.candidate_assignment_generations.capacity();
    let key_pointer = state.candidate_item_keys.as_ptr();
    let assignment_pointer = state.candidate_assignment_generations.as_ptr();
    assert!(key_capacity > 0);
    assert!(assignment_capacity > 0);

    surface.tree.node_mut(owner_id()).unwrap().scroll_state = Some(UiScrollState {
        offset: 72.0,
        ..scroll_state()
    });
    surface
        .reconcile_virtual_list_materialization(owner_id(), 100_000, &mut changes)
        .unwrap();

    let state = surface
        .virtual_list_materialization
        .owners
        .get(&owner_id())
        .expect("reconciliation retains owner state");
    assert_eq!(state.candidate_item_keys.capacity(), key_capacity);
    assert_eq!(
        state.candidate_assignment_generations.capacity(),
        assignment_capacity
    );
    assert!(
        state.slot_item_keys.as_ptr() == key_pointer
            || state.candidate_item_keys.as_ptr() == key_pointer
    );
    assert!(
        state.slot_assignment_generations.as_ptr() == assignment_pointer
            || state.candidate_assignment_generations.as_ptr() == assignment_pointer
    );
}

#[test]
fn cloned_materialization_drops_candidate_scratch() {
    let mut surface = surface_with_owner(true);
    let mut changes = Vec::new();
    for _ in 0..3 {
        surface
            .reconcile_virtual_list_materialization(owner_id(), 100_000, &mut changes)
            .unwrap();
    }

    let cloned = surface.virtual_list_materialization.clone();
    let state = cloned
        .owners
        .get(&owner_id())
        .expect("clone retains published owner assignment");
    assert!(state.candidate_item_keys.is_empty());
    assert!(state.candidate_assignment_generations.is_empty());
    assert!(state.planner_changes.is_empty());
    assert_eq!(state.candidate_slots.slot_count(), 0);
    assert_eq!(state.slots.slot_count(), 41);
}

#[test]
fn oversized_physical_slot_request_is_rejected_before_state_mutation() {
    let mut surface = surface_with_owner(true);
    let mut changes = Vec::new();
    surface
        .reconcile_virtual_list_materialization(owner_id(), 1_000_000, &mut changes)
        .unwrap();
    let before = surface.virtual_list_slot_map(owner_id()).unwrap().clone();
    let UiContainerKind::ScrollableBox(config) =
        &mut surface.tree.node_mut(owner_id()).unwrap().container
    else {
        unreachable!("fixture owner must be scrollable");
    };
    config.virtualization.as_mut().unwrap().overscan = usize::MAX;

    let mut key_callback_count = 0;
    let error = surface
        .reconcile_virtual_list_materialization_with_keys(
            owner_id(),
            1_000_000,
            &mut changes,
            |logical_index| {
                key_callback_count += 1;
                UiVirtualListItemKey::new(logical_index as u128)
            },
        )
        .unwrap_err();

    assert_eq!(
        error,
        UiVirtualListMaterializationError::SlotCapacityExceeded {
            owner_id: owner_id(),
            requested: 1_000_000,
            maximum: crate::ui::layout::MAX_UI_LAYOUT_DISCRETE_VALUE,
        }
    );
    assert_eq!(key_callback_count, 0);
    assert!(changes.is_empty());
    assert_eq!(surface.virtual_list_slot_map(owner_id()).unwrap(), &before);
}

#[test]
fn physical_slot_budget_covers_extent_ratio_and_boundary() {
    let logical_count = 1_000_000;
    let mut at_budget = surface_with_owner(true);
    let UiContainerKind::ScrollableBox(config) =
        &mut at_budget.tree.node_mut(owner_id()).unwrap().container
    else {
        unreachable!("fixture owner must be scrollable");
    };
    config.virtualization = Some(UiVirtualListConfig {
        item_extent: 1.0,
        overscan: 0,
    });
    at_budget.tree.node_mut(owner_id()).unwrap().scroll_state = Some(UiScrollState {
        viewport_extent: 4_095.0,
        ..scroll_state()
    });
    let report = at_budget
        .reconcile_virtual_list_materialization(owner_id(), logical_count, &mut Vec::new())
        .unwrap();
    assert_eq!(
        report.slot_capacity,
        crate::ui::layout::MAX_UI_LAYOUT_DISCRETE_VALUE
    );

    at_budget.tree.node_mut(owner_id()).unwrap().scroll_state = Some(UiScrollState {
        viewport_extent: 4_096.0,
        ..scroll_state()
    });
    let error = at_budget
        .reconcile_virtual_list_materialization(owner_id(), logical_count, &mut Vec::new())
        .unwrap_err();
    assert_eq!(
        error,
        UiVirtualListMaterializationError::SlotCapacityExceeded {
            owner_id: owner_id(),
            requested: crate::ui::layout::MAX_UI_LAYOUT_DISCRETE_VALUE + 1,
            maximum: crate::ui::layout::MAX_UI_LAYOUT_DISCRETE_VALUE,
        }
    );

    let mut tiny_extent = surface_with_owner(true);
    let UiContainerKind::ScrollableBox(config) =
        &mut tiny_extent.tree.node_mut(owner_id()).unwrap().container
    else {
        unreachable!("fixture owner must be scrollable");
    };
    config.virtualization = Some(UiVirtualListConfig {
        item_extent: f32::MIN_POSITIVE,
        overscan: 0,
    });
    tiny_extent.tree.node_mut(owner_id()).unwrap().scroll_state = Some(UiScrollState {
        viewport_extent: f32::MAX,
        ..scroll_state()
    });
    assert!(matches!(
        tiny_extent.reconcile_virtual_list_materialization(
            owner_id(),
            logical_count,
            &mut Vec::new()
        ),
        Err(UiVirtualListMaterializationError::SlotCapacityExceeded { .. })
    ));
}

#[test]
fn non_finite_virtual_viewports_materialize_no_slots() {
    for viewport_extent in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut surface = surface_with_owner(true);
        surface.tree.node_mut(owner_id()).unwrap().scroll_state = Some(UiScrollState {
            viewport_extent,
            ..scroll_state()
        });

        let report = surface
            .reconcile_virtual_list_materialization(owner_id(), 1_000_000, &mut Vec::new())
            .unwrap();

        assert_eq!(report.slot_capacity, 0);
        assert_eq!(report.active_slot_count, 0);
    }
}

#[test]
fn removed_owner_state_is_pruned_without_scanning_logical_rows() {
    let mut surface = surface_with_owner(true);
    let mut changes = Vec::new();
    surface
        .reconcile_virtual_list_materialization(owner_id(), 100_000, &mut changes)
        .unwrap();
    surface.tree.nodes.remove(&owner_id());
    surface.tree.roots.clear();

    let removed = surface.prune_removed_virtual_list_materialization_owners();

    assert_eq!(removed, 1);
    assert!(surface.virtual_list_slot_map(owner_id()).is_none());
}

#[test]
fn invalidated_owner_evicts_assignments_and_clears_reused_changes() {
    let mut surface = surface_with_owner(true);
    let mut changes = Vec::new();
    surface
        .reconcile_virtual_list_materialization(owner_id(), 100_000, &mut changes)
        .unwrap();
    surface.tree.node_mut(owner_id()).unwrap().container = UiContainerKind::default();

    let error = surface
        .reconcile_virtual_list_materialization(owner_id(), 100_000, &mut changes)
        .unwrap_err();

    assert_eq!(
        error,
        UiVirtualListMaterializationError::NotVirtualizedOwner {
            owner_id: owner_id()
        }
    );
    assert!(changes.is_empty());
    assert!(surface.virtual_list_slot_map(owner_id()).is_none());
}

#[test]
fn descendant_binding_resolves_through_registered_slot() {
    let mut surface = surface_with_owner(true);
    let mut changes = Vec::new();
    let report = surface
        .reconcile_virtual_list_materialization(owner_id(), 100_000, &mut changes)
        .unwrap();
    let (slot_roots, descendants) = install_slot_subtrees(&mut surface, report.slot_capacity);
    surface
        .register_virtual_list_slots(owner_id(), &slot_roots)
        .unwrap();

    let slot_index = 7;
    let binding = surface
        .virtual_list_binding_for_node(owner_id(), descendants[slot_index])
        .unwrap();

    assert_eq!(binding.owner_id, owner_id());
    assert_eq!(binding.slot_index, slot_index);
    assert_eq!(binding.slot_root_id, slot_roots[slot_index]);
    assert_eq!(
        binding.item_key,
        UiVirtualListItemKey::new(binding.logical_index as u128)
    );
    assert_eq!(
        Some(binding.logical_index),
        surface
            .virtual_list_slot_map(owner_id())
            .unwrap()
            .logical_index_for_slot(slot_index)
    );
}

#[test]
fn captured_slot_rebind_is_rejected_before_assignment_commit() {
    let mut surface = surface_with_owner(true);
    surface.tree.node_mut(owner_id()).unwrap().scroll_state = Some(UiScrollState {
        offset: 96.0,
        ..scroll_state()
    });
    let mut changes = Vec::new();
    let report = surface
        .reconcile_virtual_list_materialization(owner_id(), 100_000, &mut changes)
        .unwrap();
    let (slot_roots, _) = install_slot_subtrees(&mut surface, report.slot_capacity);
    surface
        .register_virtual_list_slots(owner_id(), &slot_roots)
        .unwrap();
    let protected_slot = 1;
    let generation = surface
        .virtual_list_slot_map(owner_id())
        .unwrap()
        .generation();
    let logical_index = surface
        .virtual_list_slot_map(owner_id())
        .unwrap()
        .logical_index_for_slot(protected_slot);
    surface
        .input
        .set_pointer_capture_for_id(UiPointerId::new(7), slot_roots[protected_slot]);
    surface.tree.node_mut(owner_id()).unwrap().scroll_state = Some(UiScrollState {
        offset: 120.0,
        ..scroll_state()
    });

    let error = surface
        .reconcile_virtual_list_materialization(owner_id(), 100_000, &mut changes)
        .unwrap_err();

    assert_eq!(
        error,
        UiVirtualListMaterializationError::ProtectedSlotRebind {
            owner_id: owner_id(),
            slot_index: protected_slot,
            node_id: slot_roots[protected_slot],
        }
    );
    assert!(changes.is_empty());
    let slots = surface.virtual_list_slot_map(owner_id()).unwrap();
    assert_eq!(slots.generation(), generation);
    assert_eq!(slots.logical_index_for_slot(protected_slot), logical_index);
}

#[test]
fn stable_item_key_follows_logical_item_across_slot_reuse() {
    let mut surface = surface_with_owner(true);
    let mut changes = Vec::new();
    let report = surface
        .reconcile_virtual_list_materialization_with_keys(
            owner_id(),
            100_000,
            &mut changes,
            |logical_index| UiVirtualListItemKey::new(10_000 + logical_index as u128),
        )
        .unwrap();
    let (slot_roots, _) = install_slot_subtrees(&mut surface, report.slot_capacity);
    surface
        .register_virtual_list_slots(owner_id(), &slot_roots)
        .unwrap();
    let slot_index = 7;
    let previous = surface
        .virtual_list_binding_for_node(owner_id(), slot_roots[slot_index])
        .unwrap();
    surface.tree.node_mut(owner_id()).unwrap().scroll_state = Some(UiScrollState {
        offset: 1_200_000.0,
        ..scroll_state()
    });

    surface
        .reconcile_virtual_list_materialization_with_keys(
            owner_id(),
            100_000,
            &mut changes,
            |logical_index| UiVirtualListItemKey::new(10_000 + logical_index as u128),
        )
        .unwrap();

    let rebound = surface
        .virtual_list_binding_for_node(owner_id(), slot_roots[slot_index])
        .unwrap();
    assert_eq!(rebound.slot_root_id, previous.slot_root_id);
    assert_ne!(rebound.logical_index, previous.logical_index);
    assert_eq!(
        rebound.item_key,
        UiVirtualListItemKey::new(10_000 + rebound.logical_index as u128)
    );
}

#[test]
fn key_only_rebind_advances_materialization_generation() {
    let mut surface = surface_with_owner(true);
    let mut changes = Vec::new();
    let first = surface
        .reconcile_virtual_list_materialization_with_keys(
            owner_id(),
            100_000,
            &mut changes,
            |logical_index| UiVirtualListItemKey::new(1_000 + logical_index as u128),
        )
        .unwrap();
    let (slot_roots, _) = install_slot_subtrees(&mut surface, first.slot_capacity);
    surface
        .register_virtual_list_slots(owner_id(), &slot_roots)
        .unwrap();

    let second = surface
        .reconcile_virtual_list_materialization_with_keys(
            owner_id(),
            100_000,
            &mut changes,
            |logical_index| {
                if logical_index == 7 {
                    UiVirtualListItemKey::new(99_999)
                } else {
                    UiVirtualListItemKey::new(1_000 + logical_index as u128)
                }
            },
        )
        .unwrap();

    assert_eq!(second.generation, first.generation + 1);
    assert_eq!(second.changed_slot_count, 1);
    assert_eq!(changes[0].logical_index, Some(7));
    assert_eq!(changes[0].item_key, Some(UiVirtualListItemKey::new(99_999)));
    assert_eq!(
        surface
            .virtual_list_binding_for_node(owner_id(), slot_roots[7])
            .unwrap()
            .item_key,
        UiVirtualListItemKey::new(99_999)
    );
}

#[test]
fn rebound_slot_rejects_its_previous_logical_identity() {
    let mut surface = surface_with_owner(true);
    surface.tree.node_mut(owner_id()).unwrap().scroll_state = Some(UiScrollState {
        offset: 96.0,
        ..scroll_state()
    });
    let mut changes = Vec::new();
    let report = surface
        .reconcile_virtual_list_materialization(owner_id(), 100_000, &mut changes)
        .unwrap();
    let (slot_roots, _) = install_slot_subtrees(&mut surface, report.slot_capacity);
    surface
        .register_virtual_list_slots(owner_id(), &slot_roots)
        .unwrap();
    let previous = slot_roots
        .iter()
        .map(|node_id| {
            surface
                .virtual_list_binding_for_node(owner_id(), *node_id)
                .unwrap()
        })
        .collect::<Vec<_>>();
    surface.tree.node_mut(owner_id()).unwrap().scroll_state = Some(UiScrollState {
        offset: 120.0,
        ..scroll_state()
    });

    surface
        .reconcile_virtual_list_materialization(owner_id(), 100_000, &mut changes)
        .unwrap();

    assert_eq!(changes.len(), 1);
    let slot_index = changes[0].slot_index;
    let current = surface
        .virtual_list_binding_for_node(owner_id(), slot_roots[slot_index])
        .unwrap();
    assert_ne!(
        current.item_identity(),
        previous[slot_index].item_identity()
    );
    assert_ne!(
        current.assignment_generation,
        previous[slot_index].assignment_generation
    );
    assert!(!surface.virtual_list_binding_is_current(slot_roots[slot_index], previous[slot_index],));
    assert!(surface.virtual_list_binding_is_current(slot_roots[slot_index], current));
}

#[test]
fn unchanged_slot_preserves_its_assignment_generation() {
    let mut surface = surface_with_owner(true);
    surface.tree.node_mut(owner_id()).unwrap().scroll_state = Some(UiScrollState {
        offset: 96.0,
        ..scroll_state()
    });
    let mut changes = Vec::new();
    let report = surface
        .reconcile_virtual_list_materialization(owner_id(), 100_000, &mut changes)
        .unwrap();
    let (slot_roots, _) = install_slot_subtrees(&mut surface, report.slot_capacity);
    surface
        .register_virtual_list_slots(owner_id(), &slot_roots)
        .unwrap();
    let previous = slot_roots
        .iter()
        .map(|node_id| {
            surface
                .virtual_list_binding_for_node(owner_id(), *node_id)
                .unwrap()
        })
        .collect::<Vec<_>>();
    surface.tree.node_mut(owner_id()).unwrap().scroll_state = Some(UiScrollState {
        offset: 120.0,
        ..scroll_state()
    });

    surface
        .reconcile_virtual_list_materialization(owner_id(), 100_000, &mut changes)
        .unwrap();

    assert_eq!(changes.len(), 1);
    let rebound_slot = changes[0].slot_index;
    let unchanged_slot = (0..slot_roots.len())
        .find(|slot_index| *slot_index != rebound_slot)
        .unwrap();
    let current = surface
        .virtual_list_binding_for_node(owner_id(), slot_roots[unchanged_slot])
        .unwrap();
    assert_eq!(current, previous[unchanged_slot]);
    assert!(surface
        .virtual_list_binding_is_current(slot_roots[unchanged_slot], previous[unchanged_slot],));
}

fn install_slot_subtrees(
    surface: &mut UiSurface,
    slot_count: usize,
) -> (Vec<UiNodeId>, Vec<UiNodeId>) {
    let mut slot_roots = Vec::with_capacity(slot_count);
    let mut descendants = Vec::with_capacity(slot_count);
    for slot_index in 0..slot_count {
        let slot_root = UiNodeId::new(10 + slot_index as u64);
        let descendant = UiNodeId::new(1_000 + slot_index as u64);
        surface
            .tree
            .insert_child(
                owner_id(),
                UiTreeNode::new(
                    slot_root,
                    UiNodePath::new(format!("root/list/slot-{slot_index}")),
                ),
            )
            .unwrap();
        surface
            .tree
            .insert_child(
                slot_root,
                UiTreeNode::new(
                    descendant,
                    UiNodePath::new(format!("root/list/slot-{slot_index}/label")),
                ),
            )
            .unwrap();
        slot_roots.push(slot_root);
        descendants.push(descendant);
    }
    (slot_roots, descendants)
}

fn surface_with_owner(virtualized: bool) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.virtual_list.surface"));
    let virtualization = virtualized.then_some(UiVirtualListConfig {
        item_extent: 24.0,
        overscan: 3,
    });
    surface.tree.insert_root(
        UiTreeNode::new(owner_id(), UiNodePath::new("root/list"))
            .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                virtualization,
                ..UiScrollableBoxConfig::default()
            }))
            .with_scroll_state(scroll_state()),
    );
    surface
}

fn scroll_state() -> UiScrollState {
    UiScrollState {
        offset: 0.0,
        viewport_extent: 800.0,
        content_extent: 2_400_000.0,
    }
}

fn owner_id() -> UiNodeId {
    UiNodeId::new(1)
}
