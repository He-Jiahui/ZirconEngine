use std::collections::BTreeMap;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::event_ui::UiTreeId;

use crate::ui::retained_host::route_intent::EditorRouteIntentMap;

use super::viewport_toolbar_pointer_bridge::ViewportToolbarPointerBridge;
use super::viewport_toolbar_pointer_layout::ViewportToolbarPointerLayout;

impl ViewportToolbarPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: ViewportToolbarPointerLayout::default(),
            controls_by_surface: BTreeMap::new(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.viewport_toolbar.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            route_intents: EditorRouteIntentMap::default(),
        };
        bridge.rebuild_surface();
        bridge
    }
}
