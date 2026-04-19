use super::{UiTree, UiTreeError};
use crate::event_ui::UiNodeId;

impl UiTree {
    pub fn mark_layout_dirty(&mut self, node_id: UiNodeId) -> Result<(), UiTreeError> {
        self.mark_layout_dirty_local(node_id)?;

        let mut current = node_id;
        while let Some(parent_id) = self
            .nodes
            .get(&current)
            .ok_or(UiTreeError::MissingNode(current))?
            .parent
        {
            self.mark_layout_dirty_local(parent_id)?;
            if !self
                .nodes
                .get(&parent_id)
                .ok_or(UiTreeError::MissingNode(parent_id))?
                .layout_boundary
                .propagates_child_layout_invalidation()
            {
                break;
            }
            current = parent_id;
        }

        Ok(())
    }

    fn mark_layout_dirty_local(&mut self, node_id: UiNodeId) -> Result<(), UiTreeError> {
        let node = self
            .nodes
            .get_mut(&node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        node.dirty.layout = true;
        node.dirty.hit_test = true;
        node.dirty.render = true;
        node.state_flags.dirty = true;
        Ok(())
    }
}
