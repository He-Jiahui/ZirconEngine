use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::event_ui::UiTreeId;

use super::host_drawer_header_pointer_bridge::HostDrawerHeaderPointerBridge;
use super::host_drawer_header_pointer_layout::HostDrawerHeaderPointerLayout;

impl HostDrawerHeaderPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: HostDrawerHeaderPointerLayout::default(),
            measured_frames: Default::default(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.drawer_header.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            route_intents: Default::default(),
            #[cfg(test)]
            surface_authority_generation: 0,
        };
        bridge.rebuild_surface();
        bridge
    }
}
