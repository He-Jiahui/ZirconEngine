use std::sync::{Arc, Mutex};

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};

use crate::scene::viewport::pointer::{
    precision::SharedResolutionState, viewport_pointer_layout::ViewportPointerLayout,
};

pub(crate) struct ViewportOverlayPointerBridge {
    pub(in crate::scene::viewport::pointer) layout: ViewportPointerLayout,
    pub(in crate::scene::viewport::pointer) surface: UiSurface,
    pub(in crate::scene::viewport::pointer) dispatcher: UiPointerDispatcher,
    pub(in crate::scene::viewport::pointer) shared: Arc<Mutex<SharedResolutionState>>,
}
