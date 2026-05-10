use std::collections::BTreeMap;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::event_ui::UiTreeId;

use super::host_activity_rail_pointer_bridge::HostActivityRailPointerBridge;
use super::host_activity_rail_pointer_layout::HostActivityRailPointerLayout;

impl HostActivityRailPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: HostActivityRailPointerLayout::default(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.activity_rail.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: BTreeMap::new(),
        };
        bridge.rebuild_surface();
        bridge
    }
}
