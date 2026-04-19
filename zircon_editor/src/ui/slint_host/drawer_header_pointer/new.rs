use std::collections::BTreeMap;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, event_ui::UiTreeId, surface::UiSurface};

use super::workbench_drawer_header_pointer_bridge::WorkbenchDrawerHeaderPointerBridge;
use super::workbench_drawer_header_pointer_layout::WorkbenchDrawerHeaderPointerLayout;

impl WorkbenchDrawerHeaderPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: WorkbenchDrawerHeaderPointerLayout::default(),
            measured_frames: BTreeMap::new(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.drawer_header.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: BTreeMap::new(),
        };
        bridge.rebuild_surface();
        bridge
    }
}
