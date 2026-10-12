use super::{
    host_document_tab_pointer_bridge::HostDocumentTabPointerBridge,
    host_document_tab_pointer_layout::HostDocumentTabPointerLayout,
};

impl HostDocumentTabPointerBridge {
    pub(crate) fn sync(&mut self, layout: HostDocumentTabPointerLayout) -> bool {
        if self.layout == layout {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.document_tab.receipt_sync_reuse_count",
                1
            );
            return false;
        }

        self.layout = layout;
        zircon_runtime::profile_counter!("editor", "ui.document_tab.receipt_sync_update_count", 1);
        true
    }
}
