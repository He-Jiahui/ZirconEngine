use super::host_page_pointer_bridge::HostPagePointerBridge;
use super::host_page_pointer_dispatch::HostPagePointerDispatch;
use super::host_page_pointer_route::HostPagePointerRoute;

impl HostPagePointerBridge {
    pub(crate) fn handle_click(
        &self,
        item_index: usize,
        close: bool,
    ) -> Result<HostPagePointerDispatch, String> {
        let item =
            self.layout.items.get(item_index).ok_or_else(|| {
                format!("Host page index {item_index} is outside the receipt layout")
            })?;
        let route = if close {
            if item.close_instance_id.is_none() {
                return Err(format!("Host page index {item_index} is not closeable"));
            }
            zircon_runtime::profile_counter!(
                "editor",
                "ui.host_page.native_close_receipt_count",
                1
            );
            HostPagePointerRoute::Close { item_index }
        } else {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.host_page.native_activate_receipt_count",
                1
            );
            HostPagePointerRoute::Activate { item_index }
        };
        Ok(HostPagePointerDispatch { route: Some(route) })
    }
}
