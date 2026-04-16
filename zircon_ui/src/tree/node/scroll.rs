use super::{UiTree, UiTreeError};
use crate::{UiNodeId, UiScrollState};

impl UiTree {
    pub fn set_scroll_offset(
        &mut self,
        node_id: UiNodeId,
        offset: f32,
    ) -> Result<bool, UiTreeError> {
        let (config, current_state, child_count, previous_window) = {
            let node = self
                .nodes
                .get(&node_id)
                .ok_or(UiTreeError::MissingNode(node_id))?;
            let crate::UiContainerKind::ScrollableBox(config) = node.container else {
                return Err(UiTreeError::NotScrollable(node_id));
            };
            (
                config,
                node.scroll_state.unwrap_or_default(),
                node.children.len(),
                node.layout_cache.virtual_window,
            )
        };

        let max_offset = (current_state.content_extent - current_state.viewport_extent).max(0.0);
        let clamped_offset = offset.max(0.0).min(max_offset);
        if (current_state.offset - clamped_offset).abs() <= f32::EPSILON {
            return Ok(false);
        }

        let next_window =
            config.virtual_window(clamped_offset, child_count, current_state.viewport_extent);
        let node = self
            .nodes
            .get_mut(&node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        node.scroll_state = Some(UiScrollState {
            offset: clamped_offset,
            viewport_extent: current_state.viewport_extent,
            content_extent: current_state.content_extent,
        });
        node.dirty.layout = true;
        node.dirty.hit_test = true;
        node.dirty.render = true;
        node.dirty.input = true;
        node.dirty.visible_range = previous_window != next_window;
        node.state_flags.dirty = true;
        Ok(true)
    }

    pub fn scroll_by(&mut self, node_id: UiNodeId, delta: f32) -> Result<bool, UiTreeError> {
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
