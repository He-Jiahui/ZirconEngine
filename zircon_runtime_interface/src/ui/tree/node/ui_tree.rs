use std::{
    cell::RefCell,
    collections::{btree_map, BTreeMap, BTreeSet},
    ops::{Deref, Index, IndexMut},
};

use serde::{Deserialize, Serialize};

use crate::ui::event_ui::{UiNodeId, UiTreeId};
use crate::ui::layout::{UiSlot, UiSlotKind};

use super::{UiTreeError, UiTreeNode};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UiTree {
    pub tree_id: UiTreeId,
    pub roots: Vec<UiNodeId>,
    pub nodes: UiTreeNodes,
    /// Parent-owned placement records for each retained parent-child edge.
    /// Older serialized trees omit this field, so deserialization defaults it empty.
    #[serde(default)]
    slots: Vec<UiSlot>,
    /// Runtime edge authority rebuilt once after deserializing the flat compatibility payload.
    #[serde(skip)]
    layout_slot_authority: RefCell<UiLayoutSlotAuthority>,
    /// Runtime-only generation for parent-child and slot-order topology.
    #[serde(skip)]
    pub(crate) layout_order_generation: u64,
    #[serde(skip)]
    pub(crate) pending_layout_order_parent_ids: BTreeSet<UiNodeId>,
}

impl UiTree {
    pub fn new(tree_id: UiTreeId) -> Self {
        Self {
            tree_id,
            roots: Vec::new(),
            nodes: UiTreeNodes::default(),
            slots: Vec::new(),
            layout_slot_authority: RefCell::default(),
            layout_order_generation: 0,
            pending_layout_order_parent_ids: BTreeSet::new(),
        }
    }

    pub fn insert_root(&mut self, mut node: UiTreeNode) {
        if self.nodes.contains_key(&node.node_id) {
            return;
        }
        node.parent = None;
        mark_structure_dirty(&mut node);
        self.roots.push(node.node_id);
        self.nodes.insert(node.node_id, node);
    }

    pub fn insert_child(
        &mut self,
        parent_id: UiNodeId,
        mut node: UiTreeNode,
    ) -> Result<(), UiTreeError> {
        if self.nodes.contains_key(&node.node_id) {
            return Err(UiTreeError::DuplicateNode(node.node_id));
        }
        if !self.nodes.contains_key(&parent_id) {
            return Err(UiTreeError::MissingParent(parent_id));
        }
        self.mark_layout_order_changed(parent_id);
        let parent = self
            .nodes
            .get_mut_preserving_paint_order(&parent_id)
            .expect("validated parent must remain present");
        mark_structure_dirty(parent);
        parent.children.push(node.node_id);
        node.parent = Some(parent_id);
        mark_structure_dirty(&mut node);
        self.nodes.insert(node.node_id, node);
        Ok(())
    }

    pub fn node(&self, node_id: UiNodeId) -> Option<&UiTreeNode> {
        self.nodes.get(&node_id)
    }

    /// Runtime identity for one live node allocation inside this tree instance.
    ///
    /// The value stays stable across property, layout, and sibling topology changes. Removing and
    /// reinserting the same `UiNodeId` assigns a new value, so retained-node consumers can reject
    /// stale compare-and-swap keys without invalidating unrelated owners.
    pub fn node_incarnation(&self, node_id: UiNodeId) -> Option<u64> {
        self.nodes.get(&node_id).map(|node| node.paint_order)
    }

    pub fn node_mut(&mut self, node_id: UiNodeId) -> Option<&mut UiTreeNode> {
        self.nodes.get_mut(&node_id)
    }

    pub fn layout_order_generation(&self) -> u64 {
        self.layout_order_generation
    }

    pub fn pending_layout_order_parent_ids(&self) -> &BTreeSet<UiNodeId> {
        &self.pending_layout_order_parent_ids
    }

    pub fn layout_slots(&self) -> &[UiSlot] {
        &self.slots
    }

    pub fn layout_slot(&self, slot_index: usize) -> Option<&UiSlot> {
        self.slots.get(slot_index)
    }

    pub fn first_layout_slot_index_for_edge(
        &self,
        parent_id: UiNodeId,
        child_id: UiNodeId,
    ) -> Option<usize> {
        self.ensure_layout_slot_authority();
        self.layout_slot_authority
            .borrow()
            .first_index(parent_id, child_id)
    }

    pub fn layout_slot_index_for_edge_kind(
        &self,
        parent_id: UiNodeId,
        child_id: UiNodeId,
        kind: UiSlotKind,
    ) -> Option<usize> {
        self.ensure_layout_slot_authority();
        self.layout_slot_authority
            .borrow()
            .index_for_kind(&self.slots, parent_id, child_id, kind)
    }

    pub fn push_layout_slot(&mut self, slot: UiSlot) {
        let parent_id = slot.parent_id;
        let slot_index = self.slots.len();
        self.slots.push(slot);
        self.layout_slot_authority
            .get_mut()
            .insert_if_initialized(&self.slots, slot_index);
        self.mark_layout_order_changed(parent_id);
    }

    pub fn replace_layout_slots(&mut self, slots: Vec<UiSlot>) {
        let parent_ids = self
            .slots
            .iter()
            .chain(&slots)
            .map(|slot| slot.parent_id)
            .collect::<BTreeSet<_>>();
        self.slots = slots;
        self.layout_slot_authority.get_mut().rebuild(&self.slots);
        for parent_id in parent_ids {
            self.mark_layout_order_changed(parent_id);
        }
    }

    pub fn retain_layout_slots(&mut self, mut retain: impl FnMut(&UiSlot) -> bool) {
        let mut removed_parent_ids = BTreeSet::new();
        let previous_slot_count = self.slots.len();
        self.slots.retain(|slot| {
            let retained = retain(slot);
            if !retained {
                removed_parent_ids.insert(slot.parent_id);
            }
            retained
        });
        if self.slots.len() != previous_slot_count
            && self.layout_slot_authority.get_mut().initialized
        {
            self.layout_slot_authority.get_mut().rebuild(&self.slots);
        }
        for parent_id in removed_parent_ids {
            self.mark_layout_order_changed(parent_id);
        }
    }

    pub fn mutate_layout_slot(
        &mut self,
        slot_index: usize,
        mutate: impl FnOnce(&mut UiSlot),
    ) -> Option<()> {
        let previous = self.slots.get(slot_index)?.clone();
        mutate(
            self.slots
                .get_mut(slot_index)
                .expect("validated layout slot"),
        );
        let current = self
            .slots
            .get(slot_index)
            .expect("mutated layout slot remains present");
        if previous.parent_id != current.parent_id || previous.child_id != current.child_id {
            self.layout_slot_authority.get_mut().rebind_if_initialized(
                &self.slots,
                slot_index,
                (previous.parent_id, previous.child_id),
            );
        }
        let order_changed = layout_order_slot_changed(&previous, current);
        let current_parent_id = current.parent_id;
        if order_changed {
            self.mark_layout_order_changed(previous.parent_id);
            if current_parent_id != previous.parent_id {
                self.mark_layout_order_changed(current_parent_id);
            }
        }
        Some(())
    }

    pub fn mark_layout_order_changed(&mut self, parent_id: UiNodeId) {
        self.layout_order_generation = next_layout_order_generation(self.layout_order_generation);
        self.pending_layout_order_parent_ids.insert(parent_id);
        self.nodes.mark_layout_dirty_source(parent_id);
    }

    pub fn pending_mutation_node_ids(&self) -> &BTreeSet<UiNodeId> {
        self.nodes.pending_mutation_node_ids()
    }

    pub fn pending_layout_source_node_ids(&self) -> &BTreeSet<UiNodeId> {
        self.nodes.pending_layout_source_node_ids()
    }

    pub fn clear_pending_mutation_node_ids(&mut self) {
        self.nodes.clear_pending_mutation_node_ids();
        self.pending_layout_order_parent_ids.clear();
    }

    fn ensure_layout_slot_authority(&self) {
        let needs_rebuild = {
            let authority = self.layout_slot_authority.borrow();
            !authority.initialized || authority.slot_count != self.slots.len()
        };
        if needs_rebuild {
            self.layout_slot_authority.borrow_mut().rebuild(&self.slots);
        }
    }

    #[cfg(test)]
    fn layout_slot_authority_rebuild_count(&self) -> usize {
        self.layout_slot_authority.borrow().rebuild_count
    }
}

#[derive(Clone, Debug, Default)]
struct UiLayoutSlotAuthority {
    initialized: bool,
    slot_count: usize,
    indices_by_edge: BTreeMap<(UiNodeId, UiNodeId), Vec<usize>>,
    #[cfg(test)]
    rebuild_count: usize,
}

impl UiLayoutSlotAuthority {
    fn rebuild(&mut self, slots: &[UiSlot]) {
        self.indices_by_edge.clear();
        for (slot_index, slot) in slots.iter().enumerate() {
            self.indices_by_edge
                .entry((slot.parent_id, slot.child_id))
                .or_default()
                .push(slot_index);
        }
        self.slot_count = slots.len();
        self.initialized = true;
        #[cfg(test)]
        {
            self.rebuild_count = self.rebuild_count.saturating_add(1);
        }
    }

    fn insert_if_initialized(&mut self, slots: &[UiSlot], slot_index: usize) {
        if !self.initialized {
            return;
        }
        let slot = &slots[slot_index];
        let indices = self
            .indices_by_edge
            .entry((slot.parent_id, slot.child_id))
            .or_default();
        let insert_at = indices
            .binary_search(&slot_index)
            .unwrap_or_else(|index| index);
        indices.insert(insert_at, slot_index);
        self.slot_count = slots.len();
    }

    fn rebind_if_initialized(
        &mut self,
        slots: &[UiSlot],
        slot_index: usize,
        previous_edge: (UiNodeId, UiNodeId),
    ) {
        if !self.initialized {
            return;
        }
        if let Some(indices) = self.indices_by_edge.get_mut(&previous_edge) {
            indices.retain(|index| *index != slot_index);
            if indices.is_empty() {
                self.indices_by_edge.remove(&previous_edge);
            }
        }
        let slot = &slots[slot_index];
        self.indices_by_edge
            .entry((slot.parent_id, slot.child_id))
            .or_default()
            .push(slot_index);
    }

    fn first_index(&self, parent_id: UiNodeId, child_id: UiNodeId) -> Option<usize> {
        self.indices_by_edge
            .get(&(parent_id, child_id))?
            .first()
            .copied()
    }

    fn index_for_kind(
        &self,
        slots: &[UiSlot],
        parent_id: UiNodeId,
        child_id: UiNodeId,
        kind: UiSlotKind,
    ) -> Option<usize> {
        self.indices_by_edge
            .get(&(parent_id, child_id))?
            .iter()
            .copied()
            .find(|slot_index| slots[*slot_index].kind == kind)
    }
}

fn layout_order_slot_changed(previous: &UiSlot, current: &UiSlot) -> bool {
    previous.parent_id != current.parent_id
        || previous.child_id != current.child_id
        || previous.kind != current.kind
        || previous.order != current.order
}

fn next_layout_order_generation(current: u64) -> u64 {
    let next = current.wrapping_add(1);
    if next == 0 {
        1
    } else {
        next
    }
}

/// A serialized node map whose mutable entry points retain incremental-dirty ownership.
///
/// Immutable access dereferences to `BTreeMap`; mutable access stays explicit so a caller
/// cannot change a retained node without making it a rebuild candidate.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiTreeNodes {
    nodes: BTreeMap<UiNodeId, UiTreeNode>,
    #[serde(skip)]
    mutation_node_ids: BTreeSet<UiNodeId>,
    #[serde(skip)]
    layout_source_node_ids: BTreeSet<UiNodeId>,
    #[serde(skip)]
    paint_order_cursor: PaintOrderCursor,
    #[cfg(test)]
    #[serde(skip)]
    paint_order_cursor_rebuild_node_visits: usize,
}

impl UiTreeNodes {
    pub fn get_mut(&mut self, node_id: &UiNodeId) -> Option<&mut UiTreeNode> {
        if self.nodes.contains_key(node_id) {
            self.mutation_node_ids.insert(*node_id);
            self.paint_order_cursor.invalidate();
        }
        self.nodes.get_mut(node_id)
    }

    pub fn mark_layout_dirty_source(&mut self, node_id: UiNodeId) {
        if self.nodes.contains_key(&node_id) {
            self.mutation_node_ids.insert(node_id);
            self.layout_source_node_ids.insert(node_id);
            if let Some(node) = self.nodes.get_mut(&node_id) {
                node.layout_cache.invalidate_measure();
            }
        }
    }

    pub fn insert(&mut self, node_id: UiNodeId, mut node: UiTreeNode) -> Option<UiTreeNode> {
        self.mutation_node_ids.insert(node_id);
        // Keep allocation identity and paint order on the same monotonic insertion serial.
        node.paint_order = self.allocate_paint_order();
        self.nodes.insert(node_id, node)
    }

    pub fn remove(&mut self, node_id: &UiNodeId) -> Option<UiTreeNode> {
        self.mutation_node_ids.insert(*node_id);
        self.layout_source_node_ids.remove(node_id);
        self.nodes.remove(node_id)
    }

    pub fn iter_mut(&mut self) -> btree_map::IterMut<'_, UiNodeId, UiTreeNode> {
        self.track_all_nodes();
        self.nodes.iter_mut()
    }

    pub fn values_mut(&mut self) -> btree_map::ValuesMut<'_, UiNodeId, UiTreeNode> {
        self.track_all_nodes();
        self.nodes.values_mut()
    }

    pub fn retain<F>(&mut self, predicate: F)
    where
        F: FnMut(&UiNodeId, &mut UiTreeNode) -> bool,
    {
        self.track_all_nodes();
        self.nodes.retain(predicate);
    }

    pub fn clear(&mut self) {
        self.track_all_nodes();
        self.nodes.clear();
        self.layout_source_node_ids.clear();
    }

    pub fn pending_mutation_node_ids(&self) -> &BTreeSet<UiNodeId> {
        &self.mutation_node_ids
    }

    pub fn pending_layout_source_node_ids(&self) -> &BTreeSet<UiNodeId> {
        &self.layout_source_node_ids
    }

    pub fn clear_pending_mutation_node_ids(&mut self) {
        self.mutation_node_ids.clear();
        self.layout_source_node_ids.clear();
    }

    fn track_all_nodes(&mut self) {
        self.mutation_node_ids.extend(self.nodes.keys().copied());
        self.paint_order_cursor.invalidate();
    }

    fn get_mut_preserving_paint_order(&mut self, node_id: &UiNodeId) -> Option<&mut UiTreeNode> {
        if self.nodes.contains_key(node_id) {
            self.mutation_node_ids.insert(*node_id);
        }
        self.nodes.get_mut(node_id)
    }

    fn allocate_paint_order(&mut self) -> u64 {
        if !self.paint_order_cursor.is_valid() {
            #[cfg(test)]
            {
                self.paint_order_cursor_rebuild_node_visits += self.nodes.len();
            }
            self.paint_order_cursor
                .rebuild(self.nodes.values().map(|node| node.paint_order));
        }
        self.paint_order_cursor.allocate()
    }

    #[cfg(test)]
    fn paint_order_cursor_rebuild_node_visits(&self) -> usize {
        self.paint_order_cursor_rebuild_node_visits
    }
}

impl Default for UiTreeNodes {
    fn default() -> Self {
        Self {
            nodes: BTreeMap::new(),
            mutation_node_ids: BTreeSet::new(),
            layout_source_node_ids: BTreeSet::new(),
            paint_order_cursor: PaintOrderCursor::new(),
            #[cfg(test)]
            paint_order_cursor_rebuild_node_visits: 0,
        }
    }
}

impl Deref for UiTreeNodes {
    type Target = BTreeMap<UiNodeId, UiTreeNode>;

    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}

impl Index<&UiNodeId> for UiTreeNodes {
    type Output = UiTreeNode;

    fn index(&self, node_id: &UiNodeId) -> &Self::Output {
        &self.nodes[node_id]
    }
}

impl IndexMut<&UiNodeId> for UiTreeNodes {
    fn index_mut(&mut self, node_id: &UiNodeId) -> &mut Self::Output {
        self.mutation_node_ids.insert(*node_id);
        self.paint_order_cursor.invalidate();
        self.nodes.get_mut(node_id).expect("no entry found for key")
    }
}

impl PartialEq for UiTreeNodes {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}

impl From<BTreeMap<UiNodeId, UiTreeNode>> for UiTreeNodes {
    fn from(nodes: BTreeMap<UiNodeId, UiTreeNode>) -> Self {
        let mut paint_order_cursor = PaintOrderCursor::new();
        paint_order_cursor.rebuild(nodes.values().map(|node| node.paint_order));
        Self {
            nodes,
            mutation_node_ids: BTreeSet::new(),
            layout_source_node_ids: BTreeSet::new(),
            paint_order_cursor,
            #[cfg(test)]
            paint_order_cursor_rebuild_node_visits: 0,
        }
    }
}

impl FromIterator<(UiNodeId, UiTreeNode)> for UiTreeNodes {
    fn from_iter<T: IntoIterator<Item = (UiNodeId, UiTreeNode)>>(iter: T) -> Self {
        Self::from(iter.into_iter().collect::<BTreeMap<_, _>>())
    }
}

impl<'a> IntoIterator for &'a UiTreeNodes {
    type Item = (&'a UiNodeId, &'a UiTreeNode);
    type IntoIter = btree_map::Iter<'a, UiNodeId, UiTreeNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

impl<'a> IntoIterator for &'a mut UiTreeNodes {
    type Item = (&'a UiNodeId, &'a mut UiTreeNode);
    type IntoIter = btree_map::IterMut<'a, UiNodeId, UiTreeNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.track_all_nodes();
        self.nodes.iter_mut()
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct PaintOrderCursor {
    next: u64,
    valid: bool,
}

impl PaintOrderCursor {
    const fn new() -> Self {
        Self {
            next: 0,
            valid: true,
        }
    }

    const fn is_valid(self) -> bool {
        self.valid
    }

    fn allocate(&mut self) -> u64 {
        debug_assert!(self.valid, "paint-order cursor must be rebuilt before use");
        let next = self.next;
        self.next = next.saturating_add(1);
        next
    }

    fn rebuild(&mut self, paint_orders: impl Iterator<Item = u64>) {
        let observed_next = paint_orders
            .max()
            .map_or(0, |paint_order| paint_order.saturating_add(1));
        self.next = self.next.max(observed_next);
        self.valid = true;
    }

    fn invalidate(&mut self) {
        self.valid = false;
    }
}

impl PartialEq for UiTree {
    fn eq(&self, other: &Self) -> bool {
        self.tree_id == other.tree_id
            && self.roots == other.roots
            && self.nodes == other.nodes
            && self.slots == other.slots
    }
}

fn mark_structure_dirty(node: &mut UiTreeNode) {
    node.dirty.layout = true;
    node.dirty.hit_test = true;
    node.dirty.render = true;
    node.dirty.input = true;
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::hint::black_box;
    use std::time::Instant;

    use serde_json;

    use super::{mark_structure_dirty, UiTree};
    use crate::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
    use crate::ui::layout::{UiSlot, UiSlotKind};
    use crate::ui::tree::UiTreeNode;

    #[test]
    fn bulk_insert_assigns_dense_paint_order_without_rescanning_existing_nodes() {
        const NODE_COUNT: u64 = 10_000;
        let mut tree = UiTree::new(UiTreeId::new("paint-order.bulk"));

        for index in 0..NODE_COUNT {
            tree.insert_root(node(index));
        }

        assert_eq!(tree.nodes.len(), NODE_COUNT as usize);
        assert_eq!(tree.node(UiNodeId::new(0)).unwrap().paint_order, 0);
        assert_eq!(
            tree.node(UiNodeId::new(NODE_COUNT - 1))
                .unwrap()
                .paint_order,
            NODE_COUNT - 1
        );
        assert_eq!(tree.nodes.paint_order_cursor_rebuild_node_visits(), 0);
    }

    #[test]
    fn bulk_child_insert_preserves_the_cursor_while_mutating_the_parent() {
        const CHILD_COUNT: u64 = 10_000;
        let mut tree = UiTree::new(UiTreeId::new("paint-order.children"));
        let root_id = UiNodeId::new(0);
        tree.insert_root(node(0));

        for index in 1..=CHILD_COUNT {
            tree.insert_child(root_id, node(index)).unwrap();
        }

        assert_eq!(
            tree.node(root_id).unwrap().children.len(),
            CHILD_COUNT as usize
        );
        assert_eq!(
            tree.node(UiNodeId::new(CHILD_COUNT)).unwrap().paint_order,
            CHILD_COUNT
        );
        assert_eq!(tree.nodes.paint_order_cursor_rebuild_node_visits(), 0);
    }

    #[test]
    fn child_structure_changes_invalidate_the_parent_measurement_cache() {
        let root_id = UiNodeId::new(0);
        let mut tree = UiTree::new(UiTreeId::new("layout-cache.structure"));
        tree.insert_root(node(0));
        tree.node_mut(root_id)
            .expect("root")
            .layout_cache
            .complete_measure();
        tree.clear_pending_mutation_node_ids();

        tree.insert_child(root_id, node(1)).expect("child");

        assert!(!tree.node(root_id).expect("root").layout_cache.measure_valid);
    }

    #[test]
    fn layout_order_generation_ignores_non_order_slot_mutations() {
        let root_id = UiNodeId::new(0);
        let child_id = UiNodeId::new(1);
        let mut tree = UiTree::new(UiTreeId::new("layout-order.generation"));
        tree.insert_root(node(0));
        tree.insert_child(root_id, node(1)).expect("child");
        tree.push_layout_slot(UiSlot::new(root_id, child_id, UiSlotKind::Free));
        tree.clear_pending_mutation_node_ids();
        let stable_generation = tree.layout_order_generation();

        tree.mutate_layout_slot(0, |slot| slot.z_order = 7)
            .expect("mutate non-order slot field");

        assert_eq!(tree.layout_order_generation(), stable_generation);
        assert!(tree.pending_layout_order_parent_ids().is_empty());

        tree.mutate_layout_slot(0, |slot| slot.order = 2)
            .expect("mutate slot order");

        assert_ne!(tree.layout_order_generation(), stable_generation);
        assert_eq!(
            tree.pending_layout_order_parent_ids(),
            &BTreeSet::from([root_id])
        );
    }

    #[test]
    fn deserialized_layout_slot_authority_rebuilds_once_and_keeps_missing_edges_authoritative() {
        let parent_id = UiNodeId::new(0);
        let child_id = UiNodeId::new(1);
        let mut original = UiTree::new(UiTreeId::new("layout-slot.deserialize"));
        original.insert_root(node(0));
        original.insert_child(parent_id, node(1)).expect("child");
        original.push_layout_slot(UiSlot::new(parent_id, child_id, UiSlotKind::Linear));
        let serialized = serde_json::to_vec(&original).expect("serialize UI tree");
        let restored: UiTree = serde_json::from_slice(&serialized).expect("deserialize UI tree");

        assert_eq!(restored.layout_slot_authority_rebuild_count(), 0);
        assert_eq!(
            restored.layout_slot_index_for_edge_kind(parent_id, child_id, UiSlotKind::Linear),
            Some(0)
        );
        assert_eq!(restored.layout_slot_authority_rebuild_count(), 1);
        for missing_child in 2..=1_000 {
            assert_eq!(
                restored.layout_slot_index_for_edge_kind(
                    parent_id,
                    UiNodeId::new(missing_child),
                    UiSlotKind::Linear,
                ),
                None
            );
        }
        assert_eq!(restored.layout_slot_authority_rebuild_count(), 1);
    }

    #[test]
    fn same_cardinality_slot_rebind_updates_the_edge_authority_without_rebuilding() {
        let parent_id = UiNodeId::new(0);
        let first_child_id = UiNodeId::new(1);
        let next_child_id = UiNodeId::new(2);
        let mut tree = UiTree::new(UiTreeId::new("layout-slot.rebind"));
        tree.push_layout_slot(UiSlot::new(parent_id, first_child_id, UiSlotKind::Linear));
        assert_eq!(
            tree.layout_slot_index_for_edge_kind(parent_id, first_child_id, UiSlotKind::Linear,),
            Some(0)
        );
        assert_eq!(tree.layout_slot_authority_rebuild_count(), 1);

        tree.mutate_layout_slot(0, |slot| slot.child_id = next_child_id)
            .expect("rebind slot");

        assert_eq!(
            tree.layout_slot_index_for_edge_kind(parent_id, first_child_id, UiSlotKind::Linear,),
            None
        );
        assert_eq!(
            tree.layout_slot_index_for_edge_kind(parent_id, next_child_id, UiSlotKind::Linear),
            Some(0)
        );
        assert_eq!(tree.layout_slot_authority_rebuild_count(), 1);
    }

    #[test]
    fn slot_rebind_preserves_flat_slot_precedence_on_an_existing_edge() {
        let parent_id = UiNodeId::new(0);
        let first_child_id = UiNodeId::new(1);
        let next_child_id = UiNodeId::new(2);
        let mut tree = UiTree::new(UiTreeId::new("layout-slot.rebind-order"));
        tree.push_layout_slot(UiSlot::new(parent_id, first_child_id, UiSlotKind::Linear));
        tree.push_layout_slot(UiSlot::new(parent_id, next_child_id, UiSlotKind::Linear));
        assert_eq!(
            tree.first_layout_slot_index_for_edge(parent_id, next_child_id),
            Some(1)
        );

        tree.mutate_layout_slot(0, |slot| slot.child_id = next_child_id)
            .expect("rebind slot");

        assert_eq!(
            tree.first_layout_slot_index_for_edge(parent_id, next_child_id),
            Some(0)
        );
        assert_eq!(tree.layout_slot_authority_rebuild_count(), 1);
    }

    #[test]
    fn bulk_slot_retention_reindexes_once_and_removes_the_retired_edge() {
        let parent_id = UiNodeId::new(0);
        let retained_child_id = UiNodeId::new(1);
        let removed_child_id = UiNodeId::new(2);
        let mut tree = UiTree::new(UiTreeId::new("layout-slot.retain"));
        tree.push_layout_slot(UiSlot::new(
            parent_id,
            retained_child_id,
            UiSlotKind::Linear,
        ));
        tree.push_layout_slot(UiSlot::new(parent_id, removed_child_id, UiSlotKind::Linear));
        assert_eq!(
            tree.layout_slot_index_for_edge_kind(parent_id, removed_child_id, UiSlotKind::Linear,),
            Some(1)
        );

        tree.retain_layout_slots(|slot| slot.child_id != removed_child_id);

        assert_eq!(tree.layout_slot_authority_rebuild_count(), 2);
        assert_eq!(
            tree.layout_slot_index_for_edge_kind(parent_id, retained_child_id, UiSlotKind::Linear,),
            Some(0)
        );
        assert_eq!(
            tree.layout_slot_index_for_edge_kind(parent_id, removed_child_id, UiSlotKind::Linear,),
            None
        );

        tree.retain_layout_slots(|_| true);
        assert_eq!(tree.layout_slot_authority_rebuild_count(), 2);
    }

    #[test]
    fn deserialized_tree_rebuilds_paint_order_cursor_only_once() {
        const EXISTING_NODE_COUNT: u64 = 4_096;
        let mut original = UiTree::new(UiTreeId::new("paint-order.deserialize"));
        for index in 0..EXISTING_NODE_COUNT {
            original.insert_root(node(index));
        }
        let serialized = serde_json::to_vec(&original).expect("serialize UI tree");
        let mut restored: UiTree =
            serde_json::from_slice(&serialized).expect("deserialize UI tree");

        restored.insert_root(node(EXISTING_NODE_COUNT));
        restored.insert_root(node(EXISTING_NODE_COUNT + 1));

        assert_eq!(
            restored
                .node(UiNodeId::new(EXISTING_NODE_COUNT))
                .unwrap()
                .paint_order,
            EXISTING_NODE_COUNT
        );
        assert_eq!(
            restored
                .node(UiNodeId::new(EXISTING_NODE_COUNT + 1))
                .unwrap()
                .paint_order,
            EXISTING_NODE_COUNT + 1
        );
        assert_eq!(
            restored.nodes.paint_order_cursor_rebuild_node_visits(),
            EXISTING_NODE_COUNT as usize
        );
    }

    #[test]
    fn mutable_node_access_invalidates_the_paint_order_cursor() {
        let mut tree = UiTree::new(UiTreeId::new("paint-order.mutation"));
        for index in 0..3 {
            tree.insert_root(node(index));
        }
        tree.node_mut(UiNodeId::new(1)).unwrap().paint_order = 40;

        tree.insert_root(node(3));
        tree.insert_root(node(4));

        assert_eq!(tree.node(UiNodeId::new(3)).unwrap().paint_order, 41);
        assert_eq!(tree.node(UiNodeId::new(4)).unwrap().paint_order, 42);
        assert_eq!(tree.nodes.paint_order_cursor_rebuild_node_visits(), 3);
    }

    #[test]
    fn cursor_rebuild_does_not_reuse_a_retired_high_water_order() {
        let mut tree = UiTree::new(UiTreeId::new("paint-order.retired"));
        tree.insert_root(node(0));
        tree.insert_root(node(1));
        tree.node_mut(UiNodeId::new(0)).unwrap().dirty.layout = false;
        tree.nodes.remove(&UiNodeId::new(1));

        tree.insert_root(node(2));

        assert_eq!(tree.node(UiNodeId::new(2)).unwrap().paint_order, 2);
        assert_eq!(tree.nodes.paint_order_cursor_rebuild_node_visits(), 1);
    }

    #[test]
    fn clearing_nodes_does_not_reuse_a_retired_node_incarnation() {
        let mut tree = UiTree::new(UiTreeId::new("node-incarnation.clear"));
        tree.insert_root(node(0));
        let retired = tree.node_incarnation(UiNodeId::new(0)).unwrap();

        tree.roots.clear();
        tree.nodes.clear();
        tree.insert_root(node(0));

        assert!(tree.node_incarnation(UiNodeId::new(0)).unwrap() > retired);
    }

    #[test]
    #[ignore = "release-only paint-order performance evidence"]
    fn paint_order_cursor_release_benchmark_evidence() {
        const NODE_COUNT: u64 = 10_000;
        const SAMPLE_PAIRS: usize = 21;
        let mut legacy_micros = Vec::with_capacity(SAMPLE_PAIRS);
        let mut cursor_micros = Vec::with_capacity(SAMPLE_PAIRS);

        for sample_index in 0..SAMPLE_PAIRS {
            let mut measure_legacy = || {
                let started = Instant::now();
                let mut legacy_nodes = BTreeMap::<UiNodeId, UiTreeNode>::new();
                let mut legacy_roots = Vec::with_capacity(NODE_COUNT as usize);
                for index in 0..NODE_COUNT {
                    let paint_order = legacy_nodes
                        .values()
                        .map(|node| node.paint_order)
                        .max()
                        .map_or(0, |paint_order| paint_order.saturating_add(1));
                    let mut node = node(index);
                    node.paint_order = paint_order;
                    mark_structure_dirty(&mut node);
                    legacy_roots.push(node.node_id);
                    legacy_nodes.insert(node.node_id, node);
                }
                black_box((&legacy_nodes, &legacy_roots));
                legacy_micros.push(started.elapsed().as_micros());
            };
            let mut measure_cursor = || {
                let started = Instant::now();
                let mut tree = UiTree::new(UiTreeId::new("paint-order.benchmark"));
                for index in 0..NODE_COUNT {
                    tree.insert_root(node(index));
                }
                black_box(&tree);
                cursor_micros.push(started.elapsed().as_micros());
                assert_eq!(tree.nodes.paint_order_cursor_rebuild_node_visits(), 0);
            };
            if sample_index % 2 == 0 {
                measure_legacy();
                measure_cursor();
            } else {
                measure_cursor();
                measure_legacy();
            }
        }

        let legacy_csv = legacy_micros
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let cursor_csv = cursor_micros
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let legacy_p95_us = nearest_rank_percentile(&legacy_micros, 95);
        let cursor_p95_us = nearest_rank_percentile(&cursor_micros, 95);
        println!(
            "UI_TREE_PAINT_ORDER_BENCH_V1 node_count={NODE_COUNT} sample_pairs={SAMPLE_PAIRS} legacy_scan_visits=49995000 cursor_scan_visits=0 legacy_p95_us={legacy_p95_us} cursor_p95_us={cursor_p95_us} legacy_us={legacy_csv} cursor_us={cursor_csv}"
        );
        assert!(
            cursor_p95_us.saturating_mul(4) <= legacy_p95_us,
            "cursor P95 {cursor_p95_us}us must be at most 25% of legacy P95 {legacy_p95_us}us"
        );
    }

    fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
        assert!(!samples.is_empty());
        assert!((1..=100).contains(&percentile));
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        let index = (ordered.len() * percentile).div_ceil(100) - 1;
        ordered[index]
    }

    fn node(index: u64) -> UiTreeNode {
        UiTreeNode::new(
            UiNodeId::new(index),
            UiNodePath::new(format!("root/{index}")),
        )
    }
}
