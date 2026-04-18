use super::{
    workbench_document_tab_pointer_bridge::WorkbenchDocumentTabPointerBridge,
    workbench_document_tab_pointer_layout::WorkbenchDocumentTabPointerLayout,
};

impl WorkbenchDocumentTabPointerBridge {
    pub(crate) fn sync(&mut self, layout: WorkbenchDocumentTabPointerLayout) {
        self.layout = layout;
        self.measured_frames = self
            .layout
            .surfaces
            .iter()
            .map(|surface| (surface.key.clone(), vec![None; surface.items.len()]))
            .collect();
        self.rebuild_surface();
    }
}
