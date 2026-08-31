use super::host_document_tab_pointer_bridge::HostDocumentTabPointerBridge;
use super::host_document_tab_pointer_dispatch::HostDocumentTabPointerDispatch;

impl HostDocumentTabPointerBridge {
    pub(crate) fn handle_close_click(
        &self,
        surface_key: &str,
        item_index: usize,
    ) -> Result<HostDocumentTabPointerDispatch, String> {
        zircon_runtime::profile_counter!("editor", "ui.document_tab.native_close_receipt_count", 1);
        let route = self.route_for_receipt(surface_key, item_index, true)?;
        Ok(HostDocumentTabPointerDispatch { route: Some(route) })
    }
}
