use std::collections::BTreeMap;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};

use crate::ui::retained_host::route_intent::EditorRouteIntentMap;

use super::viewport_toolbar_pointer_control::ViewportToolbarPointerControl;
use super::viewport_toolbar_pointer_layout::ViewportToolbarPointerLayout;

#[derive(Default)]
pub(crate) struct ViewportToolbarPointerBridge {
    pub(super) layout: ViewportToolbarPointerLayout,
    pub(super) controls_by_surface: BTreeMap<String, Vec<ViewportToolbarPointerControl>>,
    pub(super) surface: UiSurface,
    pub(super) dispatcher: UiPointerDispatcher,
    pub(super) route_intents: EditorRouteIntentMap,
}
