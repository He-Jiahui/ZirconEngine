use super::scroll_surface_pointer_bridge::ScrollSurfacePointerBridge;

impl ScrollSurfacePointerBridge {
    pub(super) fn clamp_scroll_offset(&mut self) {
        self.state.scroll_offset = self
            .state
            .scroll_offset
            .clamp(0.0, self.layout.max_scroll_offset());
    }
}
