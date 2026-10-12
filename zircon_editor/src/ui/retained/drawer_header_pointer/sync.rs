use super::host_drawer_header_pointer_bridge::HostDrawerHeaderPointerBridge;
use super::host_drawer_header_pointer_layout::HostDrawerHeaderPointerLayout;

impl HostDrawerHeaderPointerBridge {
    pub(crate) fn sync(&mut self, layout: HostDrawerHeaderPointerLayout) -> bool {
        if self.layout == layout {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.drawer_header.receipt_sync_reuse_count",
                1
            );
            return false;
        }

        self.layout = layout;
        zircon_runtime::profile_counter!("editor", "ui.drawer_header.receipt_sync_update_count", 1);
        true
    }
}
