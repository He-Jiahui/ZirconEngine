use super::{
    host_document_tab_pointer_bridge::HostDocumentTabPointerBridge,
    host_document_tab_pointer_layout::HostDocumentTabPointerLayout,
};

impl HostDocumentTabPointerBridge {
    pub(crate) fn sync(&mut self, layout: HostDocumentTabPointerLayout) {
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
