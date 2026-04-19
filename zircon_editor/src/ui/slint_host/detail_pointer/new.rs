use zircon_ui::{dispatch::UiPointerDispatcher, event_ui::UiTreeId, UiSurface};

use super::scroll_surface_pointer_bridge::ScrollSurfacePointerBridge;
use super::scroll_surface_pointer_layout::ScrollSurfacePointerLayout;
use super::scroll_surface_pointer_state::ScrollSurfacePointerState;

impl ScrollSurfacePointerBridge {
    pub(crate) fn new(tree_id: &'static str, path_prefix: &'static str) -> Self {
        let mut bridge = Self {
            tree_id,
            path_prefix,
            layout: ScrollSurfacePointerLayout::default(),
            state: ScrollSurfacePointerState::default(),
            surface: UiSurface::new(UiTreeId::new(tree_id)),
            dispatcher: UiPointerDispatcher::default(),
        };
        bridge.rebuild_surface();
        bridge
    }
}
