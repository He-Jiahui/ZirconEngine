use super::{UiTree, UiTreeError};
use crate::event_ui::UiNodeId;

impl UiTree {
    pub(crate) fn draw_order(&self) -> Vec<UiNodeId> {
        let mut order: Vec<_> = self
            .nodes
            .values()
            .map(|node| (node.z_index, node.paint_order, node.node_id))
            .collect();
        order.sort_by_key(|entry| (entry.0, entry.1));
        order.into_iter().map(|(_, _, node_id)| node_id).collect()
    }

    pub(crate) fn is_visible_in_tree(&self, node_id: UiNodeId) -> Result<bool, UiTreeError> {
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
