use super::scroll_surface_pointer_bridge::ScrollSurfacePointerBridge;
use super::scroll_surface_pointer_layout::ScrollSurfacePointerLayout;
use super::scroll_surface_pointer_state::ScrollSurfacePointerState;

impl ScrollSurfacePointerBridge {
    pub(crate) fn sync(
        &mut self,
        layout: ScrollSurfacePointerLayout,
        state: ScrollSurfacePointerState,
    ) {
        self.layout = layout;
        self.state = state;
        self.clamp_scroll_offset();
        self.rebuild_surface();
    }
}
