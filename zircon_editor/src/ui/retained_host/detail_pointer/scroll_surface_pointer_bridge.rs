use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};

use super::scroll_surface_pointer_layout::ScrollSurfacePointerLayout;
use super::scroll_surface_pointer_state::ScrollSurfacePointerState;

pub(crate) struct ScrollSurfacePointerBridge {
    pub(super) tree_id: &'static str,
    pub(super) path_prefix: &'static str,
    pub(super) layout: ScrollSurfacePointerLayout,
    pub(super) state: ScrollSurfacePointerState,
    pub(super) surface: UiSurface,
    pub(super) dispatcher: UiPointerDispatcher,
}
