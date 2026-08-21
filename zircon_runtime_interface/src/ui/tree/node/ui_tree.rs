use std::{
    collections::{BTreeMap, BTreeSet, btree_map},
    ops::{Deref, Index, IndexMut},
};

use serde::{Deserialize, Serialize};

use crate::ui::event_ui::{UiNodeId, UiTreeId};
use crate::ui::layout::UiSlot;

use super::{UiTreeError, UiTreeNode};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UiTree {
    pub tree_id: UiTreeId,
    pub roots: Vec<UiNodeId>,
    pub nodes: UiTreeNodes,
    /// Parent-owned placement records for each retained parent-child edge.
    /// Older serialized trees omit this field, so deserialization defaults it empty.
    #[serde(default)]
    pub slots: Vec<UiSlot>,
}

impl UiTree {
    pub fn new(tree_id: UiTreeId) -> Self {
        Self {
            tree_id,
            roots: Vec::new(),
            nodes: UiTreeNodes::default(),
            slots: Vec::new(),
        }
    }

    pub fn insert_root(&mut self, mut node: UiTreeNode) {
        if self.nodes.contains_key(&node.node_id) {
            return;
        }
        node.parent = None;
        node.paint_order = self.nodes.allocate_paint_order();
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
        let paint_order = self.nodes.allocate_paint_order();
        let parent = self
            .nodes
            .get_mut_preserving_paint_order(&parent_id)
            .expect("validated parent must remain present");
        mark_structure_dirty(parent);
        parent.children.push(node.node_id);
        node.parent = Some(parent_id);
        node.paint_order = paint_order;
        mark_structure_dirty(&mut node);
        self.nodes.insert(node.node_id, node);
        Ok(())
    }

    pub fn node(&self, node_id: UiNodeId) -> Option<&UiTreeNode> {
        self.nodes.get(&node_id)
    }

    pub fn node_mut(&mut self, node_id: UiNodeId) -> Option<&mut UiTreeNode> {
        self.nodes.get_mut(&node_id)
    }

    pub fn pending_mutation_node_ids(&self) -> &BTreeSet<UiNodeId> {
        self.nodes.pending_mutation_node_ids()
    }

    pub fn clear_pending_mutation_node_ids(&mut self) {
        self.nodes.clear_pending_mutation_node_ids();
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

    pub fn insert(&mut self, node_id: UiNodeId, node: UiTreeNode) -> Option<UiTreeNode> {
        self.mutation_node_ids.insert(node_id);
        self.paint_order_cursor.observe(node.paint_order);
        self.nodes.insert(node_id, node)
    }

    pub fn remove(&mut self, node_id: &UiNodeId) -> Option<UiTreeNode> {
        self.mutation_node_ids.insert(*node_id);
        self.nodes.remove(node_id)
    }

    pub fn entry(&mut self, node_id: UiNodeId) -> btree_map::Entry<'_, UiNodeId, UiTreeNode> {
        self.mutation_node_ids.insert(node_id);
        self.paint_order_cursor.invalidate();
        self.nodes.entry(node_id)
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
        self.paint_order_cursor.reset();
    }

    pub fn pending_mutation_node_ids(&self) -> &BTreeSet<UiNodeId> {
        &self.mutation_node_ids
    }

    pub fn clear_pending_mutation_node_ids(&mut self) {
        self.mutation_node_ids.clear();
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

    fn observe(&mut self, paint_order: u64) {
        self.next = self.next.max(paint_order.saturating_add(1));
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

    fn reset(&mut self) {
        self.next = 0;
        self.valid = true;
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
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    use serde_json;

    use super::{UiTree, mark_structure_dirty};
    use crate::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
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
