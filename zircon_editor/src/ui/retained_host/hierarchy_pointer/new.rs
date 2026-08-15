use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::event_ui::UiTreeId;

use crate::ui::retained_host::route_intent::EditorRouteIntentMap;

use super::hierarchy_pointer_bridge::HierarchyPointerBridge;
use super::hierarchy_pointer_layout::HierarchyPointerLayout;
use super::hierarchy_pointer_state::HierarchyPointerState;
use super::row_metrics::current_hierarchy_row_metrics;

impl HierarchyPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: HierarchyPointerLayout::default(),
            state: HierarchyPointerState::default(),
            row_metrics: current_hierarchy_row_metrics(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.hierarchy.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            route_intents: EditorRouteIntentMap::default(),
            #[cfg(test)]
            surface_authority_generation: 0,
        };
        bridge.rebuild_surface();
        bridge
    }
}
