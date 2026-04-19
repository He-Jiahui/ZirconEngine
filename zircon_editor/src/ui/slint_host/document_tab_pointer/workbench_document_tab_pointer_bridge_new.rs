use zircon_ui::{dispatch::UiPointerDispatcher, event_ui::UiTreeId, UiSurface};

use super::{
    workbench_document_tab_pointer_bridge::WorkbenchDocumentTabPointerBridge,
    workbench_document_tab_pointer_layout::WorkbenchDocumentTabPointerLayout,
};

impl WorkbenchDocumentTabPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: WorkbenchDocumentTabPointerLayout::default(),
            measured_frames: Default::default(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.document_tab.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: Default::default(),
        };
        bridge.rebuild_surface();
        bridge
    }
}
