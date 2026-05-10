use super::scroll_surface_pointer_bridge::ScrollSurfacePointerBridge;
use super::viewport_frame::viewport_frame;

impl ScrollSurfacePointerBridge {
    pub(super) fn clamp_scroll_offset(&mut self) {
        let max_offset =
            (self.layout.content_extent - viewport_frame(&self.layout).height).max(0.0);
        self.state.scroll_offset = self.state.scroll_offset.clamp(0.0, max_offset);
    }
}
