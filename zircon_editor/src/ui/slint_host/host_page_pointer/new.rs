use std::collections::BTreeMap;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, event_ui::UiTreeId, surface::UiSurface};

use super::workbench_host_page_pointer_bridge::WorkbenchHostPagePointerBridge;
use super::workbench_host_page_pointer_layout::WorkbenchHostPagePointerLayout;

impl WorkbenchHostPagePointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: WorkbenchHostPagePointerLayout {
                strip_frame: zircon_runtime::ui::layout::UiFrame::default(),
                items: Vec::new(),
            },
            measured_frames: Vec::new(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.host_page.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: BTreeMap::new(),
        };
        bridge.rebuild_surface();
        bridge
    }
}
