use std::collections::BTreeMap;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, event_ui::UiTreeId, surface::UiSurface};

use super::viewport_toolbar_pointer_bridge::ViewportToolbarPointerBridge;
use super::viewport_toolbar_pointer_layout::ViewportToolbarPointerLayout;

impl ViewportToolbarPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: ViewportToolbarPointerLayout::default(),
            active_controls: BTreeMap::new(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.viewport_toolbar.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: BTreeMap::new(),
        };
        bridge.rebuild_surface();
        bridge
    }
}
