use super::host_page_pointer_bridge::HostPagePointerBridge;
use super::host_page_pointer_layout::HostPagePointerLayout;

impl HostPagePointerBridge {
    pub(crate) fn sync(&mut self, layout: HostPagePointerLayout) -> bool {
        if self.layout == layout {
            zircon_runtime::profile_counter!("editor", "ui.host_page.receipt_sync_reuse_count", 1);
            return false;
        }

        self.layout = layout;
        zircon_runtime::profile_counter!("editor", "ui.host_page.receipt_sync_update_count", 1);
        true
    }
}
