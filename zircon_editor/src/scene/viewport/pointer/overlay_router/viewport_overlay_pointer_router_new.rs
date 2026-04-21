use std::sync::{Arc, Mutex};

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, event_ui::UiTreeId, surface::UiSurface};

use crate::scene::viewport::pointer::{
    precision::SharedResolutionState, viewport_pointer_layout::ViewportPointerLayout,
};

use super::{build_dispatcher::build_dispatcher, ViewportOverlayPointerRouter};

impl ViewportOverlayPointerRouter {
    pub(crate) fn new() -> Self {
        let shared = Arc::new(Mutex::new(SharedResolutionState::default()));
        let dispatcher: UiPointerDispatcher = build_dispatcher(Arc::clone(&shared));
        let mut router = Self {
            layout: ViewportPointerLayout::default(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.viewport.pointer")),
            dispatcher,
            shared,
        };
        router.rebuild_surface();
        router
    }
}
