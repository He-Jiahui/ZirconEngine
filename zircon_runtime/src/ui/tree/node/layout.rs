use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};

pub trait UiRuntimeTreeLayoutExt {
    fn mark_layout_dirty(&mut self, node_id: UiNodeId) -> Result<(), UiTreeError>;
}

impl UiRuntimeTreeLayoutExt for UiTree {
    fn mark_layout_dirty(&mut self, node_id: UiNodeId) -> Result<(), UiTreeError> {
        mark_layout_dirty_local(self, node_id)?;

        let mut current = node_id;
        while let Some(parent_id) = self
            .nodes
            .get(&current)
            .ok_or(UiTreeError::MissingNode(current))?
            .parent
        {
            mark_layout_dirty_local(self, parent_id)?;
            let parent = self
                .nodes
                .get(&parent_id)
                .ok_or(UiTreeError::MissingNode(parent_id))?;
            if !(parent.layout_boundary.propagates_child_layout_invalidation()
                || parent.container.is_auto_layout_container())
            {
                break;
            }
            current = parent_id;
        }

        Ok(())
    }
}

fn mark_layout_dirty_local(tree: &mut UiTree, node_id: UiNodeId) -> Result<(), UiTreeError> {
    let node = tree
        .nodes
        .get_mut(&node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;
    node.dirty.layout = true;
    node.dirty.hit_test = true;
    node.dirty.render = true;
    node.state_flags.dirty = true;
    Ok(())
}
