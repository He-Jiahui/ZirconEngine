use super::scroll_surface_pointer_layout::ScrollSurfacePointerLayout;
use super::scroll_surface_pointer_state::ScrollSurfacePointerState;

#[derive(Default)]
pub(crate) struct ScrollSurfacePointerBridge {
    pub(super) layout: ScrollSurfacePointerLayout,
    pub(super) state: ScrollSurfacePointerState,
}
