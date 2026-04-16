use std::sync::{Arc, Mutex};

use zircon_ui::{UiPointerDispatcher, UiSurface};

use crate::editing::viewport::pointer::{
    precision::SharedResolutionState, viewport_pointer_layout::ViewportPointerLayout,
};

pub(crate) struct ViewportOverlayPointerBridge {
    pub(in crate::editing::viewport::pointer) layout: ViewportPointerLayout,
    pub(in crate::editing::viewport::pointer) surface: UiSurface,
    pub(in crate::editing::viewport::pointer) dispatcher: UiPointerDispatcher,
    pub(in crate::editing::viewport::pointer) shared: Arc<Mutex<SharedResolutionState>>,
}
