use std::sync::Arc;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::event_ui::UiTreeId;

use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::host_menu_pointer_layout::HostMenuPointerLayout;
use super::host_menu_pointer_state::HostMenuPointerState;

impl HostMenuPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: Arc::new(HostMenuPointerLayout::default()),
            state: HostMenuPointerState::default(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.workbench.menu_pointer")),
            dispatcher: UiPointerDispatcher::default(),
            route_intents: Default::default(),
            popup_menu_index: None,
            popup_items: Vec::new(),
            popup_route_indices: Default::default(),
            #[cfg(test)]
            surface_authority_generation: 0,
        };
        bridge.rebuild_surface();
        bridge
    }
}
