use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ui::event_ui::{UiNodeId, UiTreeId};
use crate::ui::layout::UiSlot;

use super::{UiTreeError, UiTreeNode};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiTree {
    pub tree_id: UiTreeId,
    pub roots: Vec<UiNodeId>,
    pub nodes: BTreeMap<UiNodeId, UiTreeNode>,
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
            nodes: BTreeMap::new(),
            slots: Vec::new(),
        }
    }

    pub fn insert_root(&mut self, mut node: UiTreeNode) {
        if self.nodes.contains_key(&node.node_id) {
            return;
        }
        node.parent = None;
        node.paint_order = self.next_paint_order();
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
        parent.children.push(node.node_id);
        node.parent = Some(parent_id);
        node.paint_order = paint_order;
        self.nodes.insert(node.node_id, node);
        Ok(())
    }

    pub fn node(&self, node_id: UiNodeId) -> Option<&UiTreeNode> {
        self.nodes.get(&node_id)
    }

    pub fn node_mut(&mut self, node_id: UiNodeId) -> Option<&mut UiTreeNode> {
        self.nodes.get_mut(&node_id)
    }

    fn next_paint_order(&self) -> u64 {
        self.nodes
            .values()
            .map(|node| node.paint_order)
            .max()
            .map_or(0, |paint_order| paint_order.saturating_add(1))
    }
}
