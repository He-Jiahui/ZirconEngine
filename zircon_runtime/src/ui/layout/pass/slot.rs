use std::{cell::RefCell, collections::BTreeMap};

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiContainerKind, UiMargin, UiSlot},
    tree::UiTree,
};

#[derive(Clone, Debug, Default)]
pub(crate) struct UiLayoutSlotIndex {
    state: RefCell<UiLayoutSlotIndexState>,
}

#[derive(Clone, Debug, Default)]
struct UiLayoutSlotIndexState {
    initialized: bool,
    slot_count: usize,
    edge_indices: BTreeMap<(UiNodeId, UiNodeId), Vec<usize>>,
}

impl UiLayoutSlotIndex {
    pub(super) fn for_tree(tree: &UiTree) -> Self {
        let index = Self::default();
        index.refresh_for_tree(tree);
        index
    }

    pub(crate) fn refresh_for_tree(&self, tree: &UiTree) {
        self.state.borrow_mut().rebuild(tree);
    }

    pub(super) fn ensure_initialized(&self, tree: &UiTree) {
        let needs_rebuild = {
            let state = self.state.borrow();
            !state.initialized || state.slot_count != tree.slots.len()
        };
        if needs_rebuild {
            self.refresh_for_tree(tree);
        }
    }

    pub(super) fn first_index_for_edge(
        &self,
        tree: &UiTree,
        parent_id: UiNodeId,
        child_id: UiNodeId,
    ) -> Option<usize> {
        self.index_for_edge_matching(tree, parent_id, child_id, |_| true)
    }

    pub(crate) fn index_for_kind(
        &self,
        tree: &UiTree,
        parent_id: UiNodeId,
        child_id: UiNodeId,
        kind: zircon_runtime_interface::ui::layout::UiSlotKind,
    ) -> Option<usize> {
        self.index_for_edge_matching(tree, parent_id, child_id, |slot| slot.kind == kind)
    }

    fn index_for_edge_matching(
        &self,
        tree: &UiTree,
        parent_id: UiNodeId,
        child_id: UiNodeId,
        predicate: impl Fn(&UiSlot) -> bool,
    ) -> Option<usize> {
        self.ensure_initialized(tree);
        let edge = (parent_id, child_id);
        {
            let state = self.state.borrow();
            if let Some(indices) = state.edge_indices.get(&edge) {
                if indices.is_empty() {
                    return None;
                }
                let valid = indices.iter().all(|index| {
                    tree.slots.get(*index).is_some_and(|slot| {
                        slot.parent_id == parent_id && slot.child_id == child_id
                    })
                });
                if valid {
                    return indices
                        .iter()
                        .copied()
                        .find(|index| tree.slots.get(*index).is_some_and(&predicate));
                }
            }
        }

        let repaired = tree
            .slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                (slot.parent_id == parent_id && slot.child_id == child_id).then_some(index)
            })
            .collect::<Vec<_>>();
        let mut state = self.state.borrow_mut();
        state.edge_indices.insert(edge, repaired.clone());
        repaired
            .into_iter()
            .find(|index| tree.slots.get(*index).is_some_and(|slot| predicate(slot)))
    }
}

impl UiLayoutSlotIndexState {
    fn rebuild(&mut self, tree: &UiTree) {
        let mut edge_indices = BTreeMap::<_, Vec<_>>::new();
        for (parent_id, parent) in &tree.nodes {
            for child_id in &parent.children {
                edge_indices.entry((*parent_id, *child_id)).or_default();
            }
        }
        for (index, slot) in tree.slots.iter().enumerate() {
            edge_indices
                .entry((slot.parent_id, slot.child_id))
                .or_default()
                .push(index);
        }
        self.edge_indices = edge_indices;
        self.slot_count = tree.slots.len();
        self.initialized = true;
    }
}

// The index is a lazily repaired derived cache and does not contribute to surface identity.
impl PartialEq for UiLayoutSlotIndex {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

pub(super) fn slot_for_container_child<'a>(
    tree: &'a UiTree,
    slot_index: &UiLayoutSlotIndex,
    parent_id: UiNodeId,
    child_id: UiNodeId,
    container: UiContainerKind,
) -> Option<&'a UiSlot> {
    let slot_kind = slot_kind_for_container(container)?;
    tree.slots
        .get(slot_index.index_for_kind(tree, parent_id, child_id, slot_kind)?)
}

pub(super) fn ordered_children_for_container(
    tree: &UiTree,
    slot_index: &UiLayoutSlotIndex,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    container: UiContainerKind,
) -> Vec<UiNodeId> {
    let mut indexed: Vec<_> = children
        .iter()
        .copied()
        .enumerate()
        .map(|(index, child_id)| {
            let order = slot_for_container_child(tree, slot_index, parent_id, child_id, container)
                .map(|slot| slot.order)
                .unwrap_or_default();
            (order, index, child_id)
        })
        .collect();
    indexed.sort_by_key(|(order, index, _)| (*order, *index));
    indexed
        .into_iter()
        .map(|(_, _, child_id)| child_id)
        .collect()
}

pub(super) fn has_slot_frame_policy(slot: Option<&UiSlot>) -> bool {
    slot.is_some_and(|slot| {
        slot.padding != UiMargin::default() || slot.alignment != Default::default()
    })
}

pub(super) fn slot_padding(slot: Option<&UiSlot>) -> UiMargin {
    slot.filter(|slot| slot.padding != UiMargin::default())
        .map(|slot| slot.padding)
        .unwrap_or_default()
}

fn slot_kind_for_container(
    container: UiContainerKind,
) -> Option<zircon_runtime_interface::ui::layout::UiSlotKind> {
    container.child_slot_kind()
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{
        event_ui::UiNodeId,
        layout::{UiContainerKind, UiSlot, UiSlotKind},
        tree::UiTree,
    };

    use super::{slot_for_container_child, UiLayoutSlotIndex};

    #[test]
    fn indexed_lookup_preserves_first_matching_slot_semantics() {
        let parent_id = UiNodeId::new(1);
        let child_id = UiNodeId::new(2);
        let mut tree = UiTree::default();
        tree.slots = vec![
            UiSlot::new(parent_id, child_id, UiSlotKind::Free).with_order(9),
            UiSlot::new(parent_id, child_id, UiSlotKind::Linear).with_order(1),
            UiSlot::new(parent_id, child_id, UiSlotKind::Linear).with_order(2),
        ];
        let slot_index = UiLayoutSlotIndex::for_tree(&tree);

        let slot = slot_for_container_child(
            &tree,
            &slot_index,
            parent_id,
            child_id,
            UiContainerKind::HorizontalBox(Default::default()),
        )
        .expect("linear slot should be indexed");

        assert_eq!(slot.order, 1);
    }

    #[test]
    fn indexed_lookup_repairs_same_cardinality_edge_replacement() {
        let parent_id = UiNodeId::new(1);
        let old_child_id = UiNodeId::new(2);
        let next_child_id = UiNodeId::new(3);
        let mut tree = UiTree::default();
        tree.slots = vec![UiSlot::new(parent_id, old_child_id, UiSlotKind::Linear)];
        let slot_index = UiLayoutSlotIndex::for_tree(&tree);

        tree.slots = vec![UiSlot::new(parent_id, next_child_id, UiSlotKind::Linear).with_order(7)];

        let slot = slot_for_container_child(
            &tree,
            &slot_index,
            parent_id,
            next_child_id,
            UiContainerKind::HorizontalBox(Default::default()),
        )
        .expect("replacement edge should repair the cached slot lookup");
        assert_eq!(slot.order, 7);
        assert!(slot_for_container_child(
            &tree,
            &slot_index,
            parent_id,
            old_child_id,
            UiContainerKind::HorizontalBox(Default::default()),
        )
        .is_none());
    }
}
