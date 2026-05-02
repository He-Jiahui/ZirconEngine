use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError, UiTreeNode};

pub trait UiRuntimeTreeAccessExt {
    fn new(tree_id: UiTreeId) -> Self;
    fn insert_root(&mut self, node: UiTreeNode);
    fn insert_child(&mut self, parent_id: UiNodeId, node: UiTreeNode) -> Result<(), UiTreeError>;
    fn node(&self, node_id: UiNodeId) -> Option<&UiTreeNode>;
    fn node_mut(&mut self, node_id: UiNodeId) -> Option<&mut UiTreeNode>;
}

impl UiRuntimeTreeAccessExt for UiTree {
    fn new(tree_id: UiTreeId) -> Self {
        Self {
            tree_id,
            roots: Vec::new(),
            nodes: Default::default(),
        }
    }

    fn insert_root(&mut self, mut node: UiTreeNode) {
        if self.nodes.contains_key(&node.node_id) {
            return;
        }
        node.parent = None;
        node.paint_order = next_paint_order(self);
        self.roots.push(node.node_id);
        self.nodes.insert(node.node_id, node);
    }

    fn insert_child(
        &mut self,
        parent_id: UiNodeId,
        mut node: UiTreeNode,
    ) -> Result<(), UiTreeError> {
        if self.nodes.contains_key(&node.node_id) {
            return Err(UiTreeError::DuplicateNode(node.node_id));
        }
        let paint_order = next_paint_order(self);
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

    fn node(&self, node_id: UiNodeId) -> Option<&UiTreeNode> {
        self.nodes.get(&node_id)
    }

    fn node_mut(&mut self, node_id: UiNodeId) -> Option<&mut UiTreeNode> {
        self.nodes.get_mut(&node_id)
    }
}

fn next_paint_order(tree: &UiTree) -> u64 {
    tree.nodes
        .values()
        .map(|node| node.paint_order)
        .max()
        .map_or(0, |paint_order| paint_order.saturating_add(1))
}
