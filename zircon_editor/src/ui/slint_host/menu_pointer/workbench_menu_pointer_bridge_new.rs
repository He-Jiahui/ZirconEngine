use zircon_ui::{dispatch::UiPointerDispatcher, event_ui::UiTreeId, UiSurface};

use super::workbench_menu_pointer_bridge::WorkbenchMenuPointerBridge;
use super::workbench_menu_pointer_layout::WorkbenchMenuPointerLayout;
use super::workbench_menu_pointer_state::WorkbenchMenuPointerState;

impl WorkbenchMenuPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: WorkbenchMenuPointerLayout::default(),
            state: WorkbenchMenuPointerState::default(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.workbench.menu_pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: Default::default(),
        };
        bridge.rebuild_surface();
        bridge
    }
}
