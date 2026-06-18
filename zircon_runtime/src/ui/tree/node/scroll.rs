use crate::ui::layout::plan_scrollable_virtual_window;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiContainerKind,
    tree::{UiTree, UiTreeError},
};

pub trait UiRuntimeTreeScrollExt {
    fn set_scroll_offset(&mut self, node_id: UiNodeId, offset: f32) -> Result<bool, UiTreeError>;
    fn scroll_by(&mut self, node_id: UiNodeId, delta: f32) -> Result<bool, UiTreeError>;
}

impl UiRuntimeTreeScrollExt for UiTree {
    fn set_scroll_offset(&mut self, node_id: UiNodeId, offset: f32) -> Result<bool, UiTreeError> {
        let (config, current_state, child_count, previous_window) = {
            let node = self
                .nodes
                .get(&node_id)
                .ok_or(UiTreeError::MissingNode(node_id))?;
            let UiContainerKind::ScrollableBox(config) = node.container else {
                return Err(UiTreeError::NotScrollable(node_id));
            };
            (
                config,
                node.scroll_state.unwrap_or_default(),
                node.children.len(),
                node.layout_cache.virtual_window,
            )
        };

        let plan = plan_scrollable_virtual_window(
            config,
            current_state,
            previous_window,
            offset,
            child_count,
            current_state.viewport_extent,
            current_state.content_extent,
        );
        if (current_state.offset - plan.scroll_state.offset).abs() <= f32::EPSILON {
            return Ok(false);
        }

        let node = self
            .nodes
            .get_mut(&node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        node.scroll_state = Some(plan.scroll_state);
        node.dirty.layout = true;
        node.dirty.hit_test = true;
        node.dirty.render = true;
        node.dirty.input = true;
        node.dirty.visible_range |= plan.visible_range_changed;
        node.state_flags.dirty = true;
        Ok(true)
    }

    fn scroll_by(&mut self, node_id: UiNodeId, delta: f32) -> Result<bool, UiTreeError> {
        let current = self
            .nodes
            .get(&node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?
            .scroll_state
            .unwrap_or_default()
            .offset;
        self.set_scroll_offset(node_id, current + delta)
    }
}
