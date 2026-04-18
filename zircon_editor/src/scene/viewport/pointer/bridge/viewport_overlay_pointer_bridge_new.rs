use std::sync::{Arc, Mutex};

use zircon_ui::{UiPointerDispatcher, UiSurface, UiTreeId};

use crate::scene::viewport::pointer::{
    bridge::build_dispatcher::build_dispatcher, precision::SharedResolutionState,
    viewport_pointer_layout::ViewportPointerLayout,
};

use super::ViewportOverlayPointerBridge;

impl ViewportOverlayPointerBridge {
    pub(crate) fn new() -> Self {
        let shared = Arc::new(Mutex::new(SharedResolutionState::default()));
        let dispatcher: UiPointerDispatcher = build_dispatcher(Arc::clone(&shared));
        let mut bridge = Self {
            layout: ViewportPointerLayout::default(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.viewport.pointer")),
            dispatcher,
            shared,
        };
        bridge.rebuild_surface();
        bridge
    }
}
