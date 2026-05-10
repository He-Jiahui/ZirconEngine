use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::event_ui::UiTreeId;

use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::host_menu_pointer_layout::HostMenuPointerLayout;
use super::host_menu_pointer_state::HostMenuPointerState;

impl HostMenuPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: HostMenuPointerLayout::default(),
            state: HostMenuPointerState::default(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.workbench.menu_pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: Default::default(),
        };
        bridge.rebuild_surface();
        bridge
    }
}
