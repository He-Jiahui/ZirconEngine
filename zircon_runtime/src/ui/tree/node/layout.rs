use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};

pub trait UiRuntimeTreeLayoutExt {
    fn mark_layout_dirty(&mut self, node_id: UiNodeId) -> Result<(), UiTreeError>;
}

impl UiRuntimeTreeLayoutExt for UiTree {
    fn mark_layout_dirty(&mut self, node_id: UiNodeId) -> Result<(), UiTreeError> {
        self.nodes.mark_layout_dirty_source(node_id);
        mark_layout_dirty_local(self, node_id, true)?;

        let mut current = node_id;
        while let Some(parent_id) = self
            .nodes
            .get(&current)
            .ok_or(UiTreeError::MissingNode(current))?
            .parent
        {
            mark_layout_dirty_local(self, parent_id, false)?;
            let parent = self
                .nodes
                .get(&parent_id)
                .ok_or(UiTreeError::MissingNode(parent_id))?;
            if !(parent
                .layout_boundary
                .propagates_child_layout_invalidation()
                || parent.container.is_auto_layout_container())
            {
                break;
            }
            current = parent_id;
        }

        Ok(())
    }
}

fn mark_layout_dirty_local(
    tree: &mut UiTree,
    node_id: UiNodeId,
    invalidate_measure: bool,
) -> Result<(), UiTreeError> {
    let node = tree
        .node_mut(node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;
    if invalidate_measure {
        node.layout_cache.invalidate_measure();
    }
    node.dirty.layout = true;
    node.dirty.hit_test = true;
    node.dirty.render = true;
    Ok(())
}
