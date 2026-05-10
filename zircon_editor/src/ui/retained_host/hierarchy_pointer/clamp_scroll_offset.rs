use super::content_height::content_height;
use super::hierarchy_pointer_bridge::HierarchyPointerBridge;
use super::viewport_frame::viewport_frame;

impl HierarchyPointerBridge {
    pub(super) fn clamp_scroll_offset(&mut self) {
        let max_offset = (content_height(self.layout.node_ids.len())
            - viewport_frame(&self.layout).height)
            .max(0.0);
        self.state.scroll_offset = self.state.scroll_offset.clamp(0.0, max_offset);
    }
}
