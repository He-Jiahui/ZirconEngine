use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;
use zircon_runtime_interface::ui::{event_ui::UiNodeId, layout::UiContainerKind, tree::UiTree};

use crate::ui::layout::{
    compute_virtual_list_window, fixed_extent_slot_capacity,
    fixed_extent_virtual_list_content_extent, fixed_extent_virtual_list_step_extent,
    UiLayoutSlotIndex, UiVirtualListSlotChange, UiVirtualListSlotMap, MAX_UI_LAYOUT_DISCRETE_VALUE,
};

use super::surface::UiSurface;

mod identity;

pub use identity::{UiVirtualListItemIdentity, UiVirtualListItemKey, UiVirtualListNodeBinding};

/// Derived, surface-local assignment state for model-backed virtual-list owners.
#[derive(Clone, Debug, Default)]
pub(super) struct UiVirtualListMaterializationIndex {
    owners: BTreeMap<UiNodeId, UiVirtualListOwnerMaterialization>,
}

#[derive(Debug, Default)]
struct UiVirtualListOwnerMaterialization {
    slots: UiVirtualListSlotMap,
    /// Reconciliation scratch retained separately so protected-slot failures leave the
    /// published assignment untouched while warm requests reuse the bounded slot buffers.
    candidate_slots: UiVirtualListSlotMap,
    slot_item_keys: Vec<Option<UiVirtualListItemKey>>,
    candidate_item_keys: Vec<Option<UiVirtualListItemKey>>,
    slot_assignment_generations: Vec<u64>,
    candidate_assignment_generations: Vec<u64>,
    planner_changes: Vec<UiVirtualListSlotChange>,
    generation: u64,
    slot_node_ids: Vec<Vec<UiNodeId>>,
    node_slots: BTreeMap<UiNodeId, usize>,
}

impl Clone for UiVirtualListOwnerMaterialization {
    fn clone(&self) -> Self {
        Self {
            slots: self.slots.clone(),
            // Candidate state is a rebuild scratch buffer, not published surface state.
            candidate_slots: UiVirtualListSlotMap::default(),
            slot_item_keys: self.slot_item_keys.clone(),
            candidate_item_keys: Vec::new(),
            slot_assignment_generations: self.slot_assignment_generations.clone(),
            candidate_assignment_generations: Vec::new(),
            // Planner changes are transaction scratch and must not be copied with published
            // assignment state when a surface snapshot is cloned.
            planner_changes: Vec::new(),
            generation: self.generation,
            slot_node_ids: self.slot_node_ids.clone(),
            node_slots: self.node_slots.clone(),
        }
    }
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
    #[error(
        "virtual-list owner {owner_id:?} requests {requested} physical slots, exceeding the maximum {maximum}"
    )]
    SlotCapacityExceeded {
        owner_id: UiNodeId,
        requested: usize,
        maximum: usize,
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
        if slot_capacity > MAX_UI_LAYOUT_DISCRETE_VALUE {
            return Err(UiVirtualListMaterializationError::SlotCapacityExceeded {
                owner_id,
                requested: slot_capacity,
                maximum: MAX_UI_LAYOUT_DISCRETE_VALUE,
            });
        }
        let state = self.owners.entry(owner_id).or_default();
        // Keep the published assignment immutable until every validation step succeeds.  The
        // scratch maps/vectors are clone-from'd in place, which preserves their bounded backing
        // storage across warm scroll requests and leaves a failed protected rebind retryable.
        state.candidate_slots.clone_from(&state.slots);
        state.candidate_slots.reconcile(
            logical_count,
            slot_capacity,
            requested_window,
            &mut state.planner_changes,
        );
        state.candidate_item_keys.clone_from(&state.slot_item_keys);
        state
            .candidate_item_keys
            .resize(state.candidate_slots.slot_count(), None);
        state
            .candidate_assignment_generations
            .clone_from(&state.slot_assignment_generations);
        state
            .candidate_assignment_generations
            .resize(state.candidate_slots.slot_count(), 0);
        for slot_index in 0..state.candidate_slots.slot_count() {
            state.candidate_item_keys[slot_index] = state
                .candidate_slots
                .logical_index_for_slot(slot_index)
                .map(&mut *item_key_for_logical_index);
        }
        changes.extend(
            (0..state
                .slot_item_keys
                .len()
                .max(state.candidate_slots.slot_count()))
                .filter_map(|slot_index| {
                    let previous_logical_index = state.slots.logical_index_for_slot(slot_index);
                    let logical_index = state.candidate_slots.logical_index_for_slot(slot_index);
                    let previous_item_key = state.slot_item_keys.get(slot_index).copied().flatten();
                    let item_key = state.candidate_item_keys.get(slot_index).copied().flatten();
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
        if state.slots.generation() != state.candidate_slots.generation() || !changes.is_empty() {
            state.generation = state.generation.wrapping_add(1);
            let generation = state.generation;
            for change in changes.iter() {
                if change.logical_index.is_some()
                    && change.slot_index < state.candidate_assignment_generations.len()
                {
                    state.candidate_assignment_generations[change.slot_index] = generation;
                }
            }
        }
        std::mem::swap(&mut state.slots, &mut state.candidate_slots);
        std::mem::swap(&mut state.slot_item_keys, &mut state.candidate_item_keys);
        std::mem::swap(
            &mut state.slot_assignment_generations,
            &mut state.candidate_assignment_generations,
        );
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
#[path = "tests/virtual_list_materialization.rs"]
mod tests;
