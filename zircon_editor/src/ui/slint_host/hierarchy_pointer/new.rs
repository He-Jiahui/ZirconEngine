use std::collections::BTreeMap;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, event_ui::UiTreeId, surface::UiSurface};

use super::hierarchy_pointer_bridge::HierarchyPointerBridge;
use super::hierarchy_pointer_layout::HierarchyPointerLayout;
use super::hierarchy_pointer_state::HierarchyPointerState;

impl HierarchyPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: HierarchyPointerLayout::default(),
            state: HierarchyPointerState::default(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.hierarchy.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: BTreeMap::new(),
        };
        bridge.rebuild_surface();
        bridge
    }
}
