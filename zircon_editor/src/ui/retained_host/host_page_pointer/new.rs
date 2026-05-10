use std::collections::BTreeMap;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::{event_ui::UiTreeId, layout::UiFrame};

use super::host_page_pointer_bridge::HostPagePointerBridge;
use super::host_page_pointer_layout::HostPagePointerLayout;

impl HostPagePointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: HostPagePointerLayout {
                strip_frame: UiFrame::default(),
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
