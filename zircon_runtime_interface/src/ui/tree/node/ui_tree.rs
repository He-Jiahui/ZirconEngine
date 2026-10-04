use std::{
    cell::RefCell,
    collections::{btree_map, BTreeMap, BTreeSet},
    ops::{Deref, Index, IndexMut},
};

use serde::{Deserialize, Serialize};

use crate::ui::event_ui::{UiNodeId, UiTreeId};
use crate::ui::layout::{UiSlot, UiSlotKind};

use super::{UiDirtyFlags, UiTreeError, UiTreeNode};

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

    /// 按需维护已初始化的端点索引；parent/child/kind/order 变化会增加父级布局顺序代次并标脏。
    /// placement、z_order 等槽载荷变化的布局或绘制失效仍由调用方按域显式传播。
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
        self.insert_if_initialized(slots, slot_index);
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
    /// Incremental dirty-domain aggregate. Mutable entry points enqueue only the affected node;
    /// the first query after deserialization performs the one required full index build.
    #[serde(skip)]
    dirty_index: RefCell<UiTreeDirtyIndex>,
    #[serde(skip)]
    dirty_index_pending_node_ids: RefCell<BTreeSet<UiNodeId>>,
    #[cfg(test)]
    #[serde(skip)]
    paint_order_cursor_rebuild_node_visits: usize,
}

#[derive(Clone, Debug, Default)]
struct UiTreeDirtyIndex {
    initialized: bool,
    node_flags: BTreeMap<UiNodeId, UiDirtyFlags>,
    domain_counts: [usize; 7],
    #[cfg(test)]
    node_visits: usize,
}

impl UiTreeDirtyIndex {
    fn update(&mut self, node_id: UiNodeId, next: UiDirtyFlags) {
        let previous = self.node_flags.get(&node_id).copied().unwrap_or_default();
        if previous == next {
            return;
        }
        self.remove_counts(previous);
        if next.any() {
            self.node_flags.insert(node_id, next);
            self.add_counts(next);
        } else {
            self.node_flags.remove(&node_id);
        }
    }

    fn flags(&self) -> UiDirtyFlags {
        UiDirtyFlags {
            layout: self.domain_counts[0] != 0,
            hit_test: self.domain_counts[1] != 0,
            render: self.domain_counts[2] != 0,
            style: self.domain_counts[3] != 0,
            text: self.domain_counts[4] != 0,
            input: self.domain_counts[5] != 0,
            visible_range: self.domain_counts[6] != 0,
        }
    }

    fn add_counts(&mut self, flags: UiDirtyFlags) {
        for (count, enabled) in self.domain_counts.iter_mut().zip([
            flags.layout,
            flags.hit_test,
            flags.render,
            flags.style,
            flags.text,
            flags.input,
            flags.visible_range,
        ]) {
            if enabled {
                *count = count.saturating_add(1);
            }
        }
    }

    fn remove_counts(&mut self, flags: UiDirtyFlags) {
        for (count, enabled) in self.domain_counts.iter_mut().zip([
            flags.layout,
            flags.hit_test,
            flags.render,
            flags.style,
            flags.text,
            flags.input,
            flags.visible_range,
        ]) {
            if enabled {
                *count = count.saturating_sub(1);
            }
        }
    }
}

impl UiTreeNodes {
    pub fn get_mut(&mut self, node_id: &UiNodeId) -> Option<&mut UiTreeNode> {
        if self.nodes.contains_key(node_id) {
            self.mark_mutation(*node_id);
            self.paint_order_cursor.invalidate();
        }
        self.nodes.get_mut(node_id)
    }

    pub fn mark_layout_dirty_source(&mut self, node_id: UiNodeId) {
        if self.nodes.contains_key(&node_id) {
            self.mark_mutation(node_id);
            self.layout_source_node_ids.insert(node_id);
            if let Some(node) = self.nodes.get_mut(&node_id) {
                node.layout_cache.invalidate_measure();
            }
        }
    }

    pub fn insert(&mut self, node_id: UiNodeId, mut node: UiTreeNode) -> Option<UiTreeNode> {
        self.mark_mutation(node_id);
        // Keep allocation identity and paint order on the same monotonic insertion serial.
        node.paint_order = self.allocate_paint_order();
        self.nodes.insert(node_id, node)
    }

    pub fn remove(&mut self, node_id: &UiNodeId) -> Option<UiTreeNode> {
        self.mark_mutation(*node_id);
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

    /// Returns the aggregate dirty domain mask, refreshing only nodes changed since the last
    /// query (or all nodes once for a newly deserialized tree).
    pub fn dirty_flags(&self) -> UiDirtyFlags {
        self.refresh_dirty_index();
        self.dirty_index.borrow().flags()
    }

    /// Returns the number of nodes with at least one effective dirty domain.
    pub fn dirty_node_count(&self) -> usize {
        self.refresh_dirty_index();
        self.dirty_index.borrow().node_flags.len()
    }

    /// Returns the currently dirty node IDs from the aggregate index.
    pub fn dirty_node_ids(&self) -> BTreeSet<UiNodeId> {
        self.refresh_dirty_index();
        self.dirty_index
            .borrow()
            .node_flags
            .keys()
            .copied()
            .collect()
    }

    /// Extends a caller-owned set with dirty node IDs without allocating an intermediate set.
    /// This is the hot-path form used when a surface already has a candidate collection ready.
    pub fn extend_dirty_node_ids(&self, target: &mut BTreeSet<UiNodeId>) {
        self.refresh_dirty_index();
        target.extend(self.dirty_index.borrow().node_flags.keys().copied());
    }

    /// Extends a caller-owned entry buffer directly from the dirty index, avoiding a temporary
    /// vector when a consumer already owns the destination used for its summary or delta.
    pub fn extend_dirty_node_entries(&self, target: &mut Vec<(UiNodeId, UiDirtyFlags)>) {
        self.refresh_dirty_index();
        let index = self.dirty_index.borrow();
        target.reserve(index.node_flags.len());
        target.extend(
            index
                .node_flags
                .iter()
                .map(|(node_id, flags)| (*node_id, *flags)),
        );
    }

    /// Returns the effective dirty domains for every currently dirty node. The index refresh is
    /// shared with `dirty_flags`/`dirty_node_count`, so repeated idle queries remain O(1) apart
    /// from the caller's requested result materialization.
    pub fn dirty_node_entries(&self) -> Vec<(UiNodeId, UiDirtyFlags)> {
        self.refresh_dirty_index();
        self.dirty_index
            .borrow()
            .node_flags
            .iter()
            .map(|(node_id, flags)| (*node_id, *flags))
            .collect()
    }

    /// Returns one node's effective dirty domains without scanning unrelated nodes.
    pub fn dirty_node_flags(&self, node_id: &UiNodeId) -> UiDirtyFlags {
        self.refresh_dirty_index();
        self.dirty_index
            .borrow()
            .node_flags
            .get(node_id)
            .copied()
            .unwrap_or_default()
    }

    pub fn clear_pending_mutation_node_ids(&mut self) {
        // Reconcile mutable references before dropping their candidate IDs. This keeps the
        // aggregate correct for callers that mutate a node and clear bookkeeping directly. A
        // newly constructed/deserialized map can defer its first full index build until a dirty
        // query; clearing mutation bookkeeping alone must not force that O(N) scan.
        let index_initialized = self.dirty_index.borrow().initialized;
        if index_initialized {
            self.refresh_dirty_index();
        }
        self.mutation_node_ids.clear();
        self.dirty_index_pending_node_ids.borrow_mut().clear();
        self.layout_source_node_ids.clear();
    }

    fn track_all_nodes(&mut self) {
        let mut dirty_index_pending_node_ids = self.dirty_index_pending_node_ids.borrow_mut();
        for node_id in self.nodes.keys().copied() {
            self.mutation_node_ids.insert(node_id);
            dirty_index_pending_node_ids.insert(node_id);
        }
        self.paint_order_cursor.invalidate();
    }

    fn get_mut_preserving_paint_order(&mut self, node_id: &UiNodeId) -> Option<&mut UiTreeNode> {
        if self.nodes.contains_key(node_id) {
            self.mark_mutation(*node_id);
        }
        self.nodes.get_mut(node_id)
    }

    fn mark_mutation(&mut self, node_id: UiNodeId) {
        self.mutation_node_ids.insert(node_id);
        self.dirty_index_pending_node_ids
            .borrow_mut()
            .insert(node_id);
    }

    fn refresh_dirty_index(&self) {
        let pending = {
            let mut pending = self.dirty_index_pending_node_ids.borrow_mut();
            let is_initialized = self.dirty_index.borrow().initialized;
            if pending.is_empty() && is_initialized {
                return;
            }
            std::mem::take(&mut *pending)
        };

        let mut index = self.dirty_index.borrow_mut();
        if !index.initialized {
            index.node_flags.clear();
            index.domain_counts = [0; 7];
            for (node_id, node) in &self.nodes {
                #[cfg(test)]
                {
                    index.node_visits = index.node_visits.saturating_add(1);
                }
                index.update(*node_id, effective_dirty_flags(node));
            }
            index.initialized = true;
            return;
        }

        for node_id in pending {
            #[cfg(test)]
            {
                index.node_visits = index.node_visits.saturating_add(1);
            }
            let dirty = self
                .nodes
                .get(&node_id)
                .map(effective_dirty_flags)
                .unwrap_or_default();
            index.update(node_id, dirty);
        }
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
    fn dirty_index_node_visits(&self) -> usize {
        self.dirty_index.borrow().node_visits
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
            dirty_index: RefCell::new(UiTreeDirtyIndex::default()),
            dirty_index_pending_node_ids: RefCell::new(BTreeSet::new()),
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
        self.mark_mutation(*node_id);
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
            dirty_index: RefCell::new(UiTreeDirtyIndex::default()),
            dirty_index_pending_node_ids: RefCell::new(BTreeSet::new()),
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

    // 只扫描当前节点会丢失已移除节点的序号；保留历史高水位可避免重新插入相同 UiNodeId 时复用退役的 node_incarnation。
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

fn effective_dirty_flags(node: &UiTreeNode) -> UiDirtyFlags {
    let mut dirty = node.dirty;
    if node.state_flags.dirty {
        dirty.hit_test = true;
        dirty.render = true;
        dirty.input = true;
    }
    dirty
}

#[cfg(test)]
#[path = "tests/ui_tree.rs"]
mod tests;
