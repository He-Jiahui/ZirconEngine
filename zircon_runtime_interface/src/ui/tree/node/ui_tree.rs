use std::{
    collections::{btree_map, BTreeMap, BTreeSet},
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
        node.paint_order = self.next_paint_order();
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
        let paint_order = self.next_paint_order();
        let parent = self
            .nodes
            .get_mut(&parent_id)
            .ok_or(UiTreeError::MissingParent(parent_id))?;
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

    fn next_paint_order(&self) -> u64 {
        self.nodes
            .values()
            .map(|node| node.paint_order)
            .max()
            .map_or(0, |paint_order| paint_order.saturating_add(1))
    }
}

/// A serialized node map whose mutable entry points retain incremental-dirty ownership.
///
/// Immutable access dereferences to `BTreeMap`; mutable access stays explicit so a caller
/// cannot change a retained node without making it a rebuild candidate.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiTreeNodes {
    nodes: BTreeMap<UiNodeId, UiTreeNode>,
    #[serde(skip)]
    mutation_node_ids: BTreeSet<UiNodeId>,
}

impl UiTreeNodes {
    pub fn get_mut(&mut self, node_id: &UiNodeId) -> Option<&mut UiTreeNode> {
        if self.nodes.contains_key(node_id) {
            self.mutation_node_ids.insert(*node_id);
        }
        self.nodes.get_mut(node_id)
    }

    pub fn insert(&mut self, node_id: UiNodeId, node: UiTreeNode) -> Option<UiTreeNode> {
        self.mutation_node_ids.insert(node_id);
        self.nodes.insert(node_id, node)
    }

    pub fn remove(&mut self, node_id: &UiNodeId) -> Option<UiTreeNode> {
        self.mutation_node_ids.insert(*node_id);
        self.nodes.remove(node_id)
    }

    pub fn entry(&mut self, node_id: UiNodeId) -> btree_map::Entry<'_, UiNodeId, UiTreeNode> {
        self.mutation_node_ids.insert(node_id);
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
    }

    pub fn pending_mutation_node_ids(&self) -> &BTreeSet<UiNodeId> {
        &self.mutation_node_ids
    }

    pub fn clear_pending_mutation_node_ids(&mut self) {
        self.mutation_node_ids.clear();
    }

    fn track_all_nodes(&mut self) {
        self.mutation_node_ids.extend(self.nodes.keys().copied());
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
        Self {
            nodes,
            mutation_node_ids: BTreeSet::new(),
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
