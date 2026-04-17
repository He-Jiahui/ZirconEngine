use std::collections::BTreeMap;

use zircon_ui::{UiPointerDispatcher, UiSurface, UiTreeId};

use super::workbench_activity_rail_pointer_bridge::WorkbenchActivityRailPointerBridge;
use super::workbench_activity_rail_pointer_layout::WorkbenchActivityRailPointerLayout;

impl WorkbenchActivityRailPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: WorkbenchActivityRailPointerLayout::default(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.activity_rail.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: BTreeMap::new(),
        };
        bridge.rebuild_surface();
        bridge
    }
}
