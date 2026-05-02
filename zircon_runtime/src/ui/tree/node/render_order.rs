use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};

pub trait UiRuntimeTreeRenderOrderExt {
    fn draw_order(&self) -> Vec<UiNodeId>;
    fn is_visible_in_tree(&self, node_id: UiNodeId) -> Result<bool, UiTreeError>;
}

impl UiRuntimeTreeRenderOrderExt for UiTree {
    fn draw_order(&self) -> Vec<UiNodeId> {
        let mut order: Vec<_> = self
            .nodes
            .values()
            .map(|node| (node.z_index, node.paint_order, node.node_id))
            .collect();
        order.sort_by_key(|entry| (entry.0, entry.1));
        order.into_iter().map(|(_, _, node_id)| node_id).collect()
    }

    fn is_visible_in_tree(&self, node_id: UiNodeId) -> Result<bool, UiTreeError> {
        let mut current = Some(node_id);
        while let Some(id) = current {
            let node = self.nodes.get(&id).ok_or(UiTreeError::MissingNode(id))?;
            if !node.state_flags.visible {
                return Ok(false);
            }
            current = node.parent;
        }
        Ok(true)
    }
}
