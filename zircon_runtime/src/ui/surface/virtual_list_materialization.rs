use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;
use zircon_runtime_interface::ui::{event_ui::UiNodeId, layout::UiContainerKind, tree::UiTree};

use crate::ui::layout::{
    UiLayoutSlotIndex, UiVirtualListSlotChange, UiVirtualListSlotMap, compute_virtual_list_window,
    fixed_extent_slot_capacity, fixed_extent_virtual_list_content_extent,
    fixed_extent_virtual_list_step_extent,
};

use super::surface::UiSurface;

mod identity;

pub use identity::{UiVirtualListItemIdentity, UiVirtualListItemKey, UiVirtualListNodeBinding};

/// Derived, surface-local assignment state for model-backed virtual-list owners.
#[derive(Clone, Debug, Default)]
pub(super) struct UiVirtualListMaterializationIndex {
    owners: BTreeMap<UiNodeId, UiVirtualListOwnerMaterialization>,
}

#[derive(Clone, Debug, Default)]
struct UiVirtualListOwnerMaterialization {
    slots: UiVirtualListSlotMap,
    slot_item_keys: Vec<Option<UiVirtualListItemKey>>,
    slot_assignment_generations: Vec<u64>,
    planner_changes: Vec<UiVirtualListSlotChange>,
    generation: u64,
    slot_node_ids: Vec<Vec<UiNodeId>>,
    node_slots: BTreeMap<UiNodeId, usize>,
}

/// Transactional physical-slot change enriched with the external model identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiVirtualListMaterializationChange {
    pub slot_index: usize,
    pub previous_logical_index: Option<usize>,
    pub logical_index: Option<usize>,
    pub previous_item_key: Option<UiVirtualListItemKey>,
    pub item_key: Option<UiVirtualListItemKey>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiVirtualListMaterializationReport {
    pub owner_id: UiNodeId,
    pub generation: u64,
    pub slot_capacity: usize,
    pub active_slot_count: usize,
    pub changed_slot_count: usize,
    pub registered_slot_count: usize,
    pub requires_slot_registration: bool,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum UiVirtualListMaterializationError {
    #[error("virtual-list materialization owner {owner_id:?} is missing")]
    MissingOwner { owner_id: UiNodeId },
    #[error("UI node {owner_id:?} is not a virtualized scroll owner")]
    NotVirtualizedOwner { owner_id: UiNodeId },
    #[error("virtualized scroll owner {owner_id:?} has no resolved scroll state")]
    MissingScrollState { owner_id: UiNodeId },
    #[error("virtual-list owner {owner_id:?} has no reconciled slot assignment")]
    MissingAssignmentState { owner_id: UiNodeId },
    #[error(
        "virtual-list owner {owner_id:?} requires {expected} physical slots but received {actual}"
    )]
    SlotCountMismatch {
        owner_id: UiNodeId,
        expected: usize,
        actual: usize,
    },
    #[error("virtual-list slot {slot_index} node {node_id:?} is not under owner {owner_id:?}")]
    InvalidSlotRoot {
        owner_id: UiNodeId,
        slot_index: usize,
        node_id: UiNodeId,
    },
    #[error("virtual-list physical node {node_id:?} is registered by multiple slots")]
    DuplicateSlotNode { node_id: UiNodeId },
    #[error(
        "virtual-list owner {owner_id:?} cannot rebind protected slot {slot_index} node {node_id:?}"
    )]
    ProtectedSlotRebind {
        owner_id: UiNodeId,
        slot_index: usize,
        node_id: UiNodeId,
    },
}

impl UiSurface {
    /// Reconciles a bounded physical-row assignment from the owner's resolved layout state.
    ///
    /// This publishes assignment authority only. Prototype subtree creation and row-model
    /// binding are separate materialization stages and must consume the returned changes.
    pub fn reconcile_virtual_list_materialization(
        &mut self,
        owner_id: UiNodeId,
        logical_count: usize,
        changes: &mut Vec<UiVirtualListMaterializationChange>,
    ) -> Result<UiVirtualListMaterializationReport, UiVirtualListMaterializationError> {
        self.reconcile_virtual_list_materialization_with_keys(
            owner_id,
            logical_count,
            changes,
            |logical_index| UiVirtualListItemKey::new(logical_index as u128),
        )
    }

    pub fn reconcile_virtual_list_materialization_with_keys(
        &mut self,
        owner_id: UiNodeId,
        logical_count: usize,
        changes: &mut Vec<UiVirtualListMaterializationChange>,
        mut item_key_for_logical_index: impl FnMut(usize) -> UiVirtualListItemKey,
    ) -> Result<UiVirtualListMaterializationReport, UiVirtualListMaterializationError> {
        let focus = &self.focus;
        let input = &self.input;
        let result = self.virtual_list_materialization.reconcile(
            &self.tree,
            owner_id,
            logical_count,
            changes,
            &mut item_key_for_logical_index,
            |node_id| {
                focus.focused == Some(node_id)
                    || focus.captured == Some(node_id)
                    || focus.pressed == Some(node_id)
                    || input
                        .pointer_captures
                        .values()
                        .any(|capture| capture.owner == node_id)
                    || input.high_precision_owner == Some(node_id)
                    || input.pointer_lock_owner == Some(node_id)
                    || input.input_method_owner == Some(node_id)
                    || input.pointer_drags.contains_key(&node_id)
                    || input
                        .drag_drop
                        .as_ref()
                        .is_some_and(|drag| drag.source == node_id || drag.target == node_id)
            },
        );
        self.virtual_list_materialization
            .publish_layout_projection(owner_id, &self.layout_slot_index);
        result
    }

    pub fn virtual_list_slot_map(&self, owner_id: UiNodeId) -> Option<&UiVirtualListSlotMap> {
        self.virtual_list_materialization.owner(owner_id)
    }

    /// Registers the bounded live row subtrees that realize an owner's physical slots.
    pub fn register_virtual_list_slots(
        &mut self,
        owner_id: UiNodeId,
        slot_root_ids: &[UiNodeId],
    ) -> Result<(), UiVirtualListMaterializationError> {
        self.virtual_list_materialization
            .register_slots(&self.tree, owner_id, slot_root_ids)?;
        self.virtual_list_materialization
            .publish_layout_projection(owner_id, &self.layout_slot_index);
        Ok(())
    }

    /// Resolves a row descendant through physical slot identity to its current logical item.
    pub fn virtual_list_binding_for_node(
        &self,
        owner_id: UiNodeId,
        node_id: UiNodeId,
    ) -> Option<UiVirtualListNodeBinding> {
        self.virtual_list_materialization
            .binding_for_node(owner_id, node_id)
    }

    /// Rejects a token captured before its physical slot was rebound to another item.
    pub fn virtual_list_binding_is_current(
        &self,
        node_id: UiNodeId,
        binding: UiVirtualListNodeBinding,
    ) -> bool {
        self.virtual_list_materialization
            .binding_for_node(binding.owner_id, node_id)
            .is_some_and(|current| current == binding)
    }

    /// Removes derived assignment state for owners no longer present in the retained tree.
    pub fn prune_removed_virtual_list_materialization_owners(&mut self) -> usize {
        let removed = self.virtual_list_materialization.prune_removed(&self.tree);
        removed
            .max(
                self.layout_slot_index
                    .prune_materialized_virtual_lists(&self.tree),
            )
            .max(self.virtual_list_prototype_pool.prune_removed(&self.tree))
    }
}

impl UiVirtualListMaterializationIndex {
    fn reconcile(
        &mut self,
        tree: &UiTree,
        owner_id: UiNodeId,
        logical_count: usize,
        changes: &mut Vec<UiVirtualListMaterializationChange>,
        item_key_for_logical_index: &mut impl FnMut(usize) -> UiVirtualListItemKey,
        mut is_protected: impl FnMut(UiNodeId) -> bool,
    ) -> Result<UiVirtualListMaterializationReport, UiVirtualListMaterializationError> {
        changes.clear();
        let Some(owner) = tree.node(owner_id) else {
            self.owners.remove(&owner_id);
            return Err(UiVirtualListMaterializationError::MissingOwner { owner_id });
        };
        let (virtualization, gap) = match owner.container {
            UiContainerKind::ScrollableBox(config) => (config.virtualization, config.gap),
            _ => (None, 0.0),
        };
        let Some(virtualization) = virtualization else {
            self.owners.remove(&owner_id);
            return Err(UiVirtualListMaterializationError::NotVirtualizedOwner { owner_id });
        };
        let Some(scroll_state) = owner.scroll_state else {
            self.owners.remove(&owner_id);
            return Err(UiVirtualListMaterializationError::MissingScrollState { owner_id });
        };
        let step_extent = fixed_extent_virtual_list_step_extent(virtualization.item_extent, gap);
        let content_extent = fixed_extent_virtual_list_content_extent(
            logical_count,
            virtualization.item_extent,
            gap,
        );
        let viewport_extent = scroll_state.viewport_extent.max(0.0);
        let requested_offset = scroll_state
            .offset
            .max(0.0)
            .min((content_extent - viewport_extent).max(0.0));
        let requested_window = compute_virtual_list_window(
            requested_offset,
            scroll_state.viewport_extent,
            step_extent,
            logical_count,
            virtualization.overscan,
        );
        let slot_capacity = fixed_extent_slot_capacity(
            scroll_state.viewport_extent,
            step_extent,
            virtualization.overscan,
            logical_count,
        );
        let state = self.owners.entry(owner_id).or_default();
        let mut candidate = state.slots.clone();
        candidate.reconcile(
            logical_count,
            slot_capacity,
            requested_window,
            &mut state.planner_changes,
        );
        let mut candidate_keys = state.slot_item_keys.clone();
        candidate_keys.resize(candidate.slot_count(), None);
        let mut candidate_assignment_generations = state.slot_assignment_generations.clone();
        candidate_assignment_generations.resize(candidate.slot_count(), 0);
        for slot_index in 0..candidate.slot_count() {
            candidate_keys[slot_index] = candidate
                .logical_index_for_slot(slot_index)
                .map(&mut *item_key_for_logical_index);
        }
        changes.extend(
            (0..state.slot_item_keys.len().max(candidate.slot_count())).filter_map(|slot_index| {
                let previous_logical_index = state.slots.logical_index_for_slot(slot_index);
                let logical_index = candidate.logical_index_for_slot(slot_index);
                let previous_item_key = state.slot_item_keys.get(slot_index).copied().flatten();
                let item_key = candidate_keys.get(slot_index).copied().flatten();
                (previous_logical_index != logical_index || previous_item_key != item_key)
                    .then_some(UiVirtualListMaterializationChange {
                        slot_index,
                        previous_logical_index,
                        logical_index,
                        previous_item_key,
                        item_key,
                    })
            }),
        );
        if let Some((slot_index, node_id)) = state.protected_rebind(changes, &mut is_protected) {
            changes.clear();
            return Err(UiVirtualListMaterializationError::ProtectedSlotRebind {
                owner_id,
                slot_index,
                node_id,
            });
        }
        if state.slots.generation() != candidate.generation() || !changes.is_empty() {
            state.generation = state.generation.wrapping_add(1);
            let generation = state.generation;
            for change in changes.iter() {
                if change.logical_index.is_some()
                    && change.slot_index < candidate_assignment_generations.len()
                {
                    candidate_assignment_generations[change.slot_index] = generation;
                }
            }
        }
        state.slots = candidate;
        state.slot_item_keys = candidate_keys;
        state.slot_assignment_generations = candidate_assignment_generations;
        let registered_slot_count = state.slot_node_ids.len();
        let requires_slot_registration = registered_slot_count != state.slots.slot_count();
        Ok(UiVirtualListMaterializationReport {
            owner_id,
            generation: state.generation,
            slot_capacity,
            active_slot_count: state.slots.active_slot_count(),
            changed_slot_count: changes.len(),
            registered_slot_count,
            requires_slot_registration,
        })
    }

    fn owner(&self, owner_id: UiNodeId) -> Option<&UiVirtualListSlotMap> {
        self.owners.get(&owner_id).map(|state| &state.slots)
    }

    fn publish_layout_projection(&self, owner_id: UiNodeId, layout_slot_index: &UiLayoutSlotIndex) {
        let Some(state) = self.owners.get(&owner_id) else {
            layout_slot_index.clear_materialized_virtual_list(owner_id);
            return;
        };
        if state.slot_node_ids.len() != state.slots.slot_count() {
            layout_slot_index.clear_materialized_virtual_list(owner_id);
            return;
        }
        let assignments =
            state
                .slot_node_ids
                .iter()
                .enumerate()
                .filter_map(|(slot_index, subtree)| {
                    Some((
                        *subtree.first()?,
                        state.slots.logical_index_for_slot(slot_index)?,
                    ))
                });
        layout_slot_index.replace_materialized_virtual_list(
            owner_id,
            state.slots.logical_count(),
            assignments,
        );
    }

    fn register_slots(
        &mut self,
        tree: &UiTree,
        owner_id: UiNodeId,
        slot_root_ids: &[UiNodeId],
    ) -> Result<(), UiVirtualListMaterializationError> {
        let state = self
            .owners
            .get_mut(&owner_id)
            .ok_or(UiVirtualListMaterializationError::MissingAssignmentState { owner_id })?;
        let expected = state.slots.slot_count();
        if slot_root_ids.len() != expected {
            return Err(UiVirtualListMaterializationError::SlotCountMismatch {
                owner_id,
                expected,
                actual: slot_root_ids.len(),
            });
        }

        let mut slot_node_ids = Vec::with_capacity(slot_root_ids.len());
        let mut node_slots = BTreeMap::new();
        for (slot_index, root_id) in slot_root_ids.iter().copied().enumerate() {
            let root =
                tree.node(root_id)
                    .ok_or(UiVirtualListMaterializationError::InvalidSlotRoot {
                        owner_id,
                        slot_index,
                        node_id: root_id,
                    })?;
            if root.parent != Some(owner_id) {
                return Err(UiVirtualListMaterializationError::InvalidSlotRoot {
                    owner_id,
                    slot_index,
                    node_id: root_id,
                });
            }
            let mut subtree = Vec::new();
            collect_subtree_node_ids(tree, owner_id, slot_index, root_id, &mut subtree)?;
            for node_id in &subtree {
                if node_slots.insert(*node_id, slot_index).is_some() {
                    return Err(UiVirtualListMaterializationError::DuplicateSlotNode {
                        node_id: *node_id,
                    });
                }
            }
            slot_node_ids.push(subtree);
        }
        state.slot_node_ids = slot_node_ids;
        state.node_slots = node_slots;
        Ok(())
    }

    fn binding_for_node(
        &self,
        owner_id: UiNodeId,
        node_id: UiNodeId,
    ) -> Option<UiVirtualListNodeBinding> {
        let state = self.owners.get(&owner_id)?;
        let slot_index = *state.node_slots.get(&node_id)?;
        let logical_index = state.slots.logical_index_for_slot(slot_index)?;
        let item_key = state.slot_item_keys.get(slot_index).copied().flatten()?;
        let assignment_generation = *state.slot_assignment_generations.get(slot_index)?;
        let slot_root_id = *state.slot_node_ids.get(slot_index)?.first()?;
        Some(UiVirtualListNodeBinding {
            owner_id,
            slot_index,
            slot_root_id,
            logical_index,
            item_key,
            assignment_generation,
        })
    }

    fn prune_removed(&mut self, tree: &UiTree) -> usize {
        let previous_count = self.owners.len();
        self.owners
            .retain(|owner_id, _| tree.nodes.contains_key(owner_id));
        previous_count - self.owners.len()
    }
}

impl UiVirtualListOwnerMaterialization {
    fn protected_rebind(
        &self,
        changes: &[UiVirtualListMaterializationChange],
        is_protected: &mut impl FnMut(UiNodeId) -> bool,
    ) -> Option<(usize, UiNodeId)> {
        changes.iter().find_map(|change| {
            (change.previous_item_key.is_some()
                && (change.previous_logical_index != change.logical_index
                    || change.previous_item_key != change.item_key))
                .then(|| self.slot_node_ids.get(change.slot_index))
                .flatten()?
                .iter()
                .copied()
                .find(|node_id| is_protected(*node_id))
                .map(|node_id| (change.slot_index, node_id))
        })
    }
}

fn collect_subtree_node_ids(
    tree: &UiTree,
    owner_id: UiNodeId,
    slot_index: usize,
    root_id: UiNodeId,
    output: &mut Vec<UiNodeId>,
) -> Result<(), UiVirtualListMaterializationError> {
    let mut pending = vec![root_id];
    let mut visited = BTreeSet::new();
    while let Some(node_id) = pending.pop() {
        if !visited.insert(node_id) {
            return Err(UiVirtualListMaterializationError::DuplicateSlotNode { node_id });
        }
        let node =
            tree.node(node_id)
                .ok_or(UiVirtualListMaterializationError::InvalidSlotRoot {
                    owner_id,
                    slot_index,
                    node_id,
                })?;
        output.push(node_id);
        pending.extend(node.children.iter().rev().copied());
    }
    Ok(())
}

// The index is a rebuildable runtime cache and does not contribute to serialized surface identity.
impl PartialEq for UiVirtualListMaterializationIndex {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
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
        assert!(
            !surface.virtual_list_binding_is_current(slot_roots[slot_index], previous[slot_index],)
        );
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
        assert!(
            surface.virtual_list_binding_is_current(
                slot_roots[unchanged_slot],
                previous[unchanged_slot],
            )
        );
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
}
