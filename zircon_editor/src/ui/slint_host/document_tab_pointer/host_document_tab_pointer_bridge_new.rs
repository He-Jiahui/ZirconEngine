use zircon_runtime::ui::{dispatch::UiPointerDispatcher, event_ui::UiTreeId, surface::UiSurface};

use super::{
    host_document_tab_pointer_bridge::HostDocumentTabPointerBridge,
    host_document_tab_pointer_layout::HostDocumentTabPointerLayout,
};

impl HostDocumentTabPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: HostDocumentTabPointerLayout::default(),
            measured_frames: Default::default(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.document_tab.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: Default::default(),
        };
        bridge.rebuild_surface();
        bridge
    }
}
