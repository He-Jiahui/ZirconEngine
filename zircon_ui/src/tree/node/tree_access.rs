use super::{UiTree, UiTreeError, UiTreeNode};
use crate::UiNodeId;

impl UiTree {
    pub fn new(tree_id: crate::UiTreeId) -> Self {
        Self {
            tree_id,
            roots: Vec::new(),
            nodes: Default::default(),
            next_paint_order: 0,
        }
    }

    pub fn insert_root(&mut self, mut node: UiTreeNode) {
        if self.nodes.contains_key(&node.node_id) {
            return;
        }
        node.parent = None;
        node.paint_order = self.next_paint_order;
        self.next_paint_order += 1;
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
        let parent = self
            .nodes
            .get_mut(&parent_id)
            .ok_or(UiTreeError::MissingParent(parent_id))?;
        parent.children.push(node.node_id);
        node.parent = Some(parent_id);
        node.paint_order = self.next_paint_order;
        self.next_paint_order += 1;
        self.nodes.insert(node.node_id, node);
        Ok(())
    }

    pub fn node(&self, node_id: UiNodeId) -> Option<&UiTreeNode> {
        self.nodes.get(&node_id)
    }

    pub fn node_mut(&mut self, node_id: UiNodeId) -> Option<&mut UiTreeNode> {
        self.nodes.get_mut(&node_id)
    }
}
