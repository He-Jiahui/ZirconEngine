use super::host_drawer_header_pointer_bridge::HostDrawerHeaderPointerBridge;
use super::host_drawer_header_pointer_dispatch::HostDrawerHeaderPointerDispatch;

impl HostDrawerHeaderPointerBridge {
    pub(crate) fn handle_click(
        &self,
        surface_key: &str,
        item_index: usize,
    ) -> Result<HostDrawerHeaderPointerDispatch, String> {
        zircon_runtime::profile_counter!("editor", "ui.drawer_header.native_receipt_count", 1);
        let route = self.route_for_receipt(surface_key, item_index)?;
        Ok(HostDrawerHeaderPointerDispatch { route: Some(route) })
    }
}
