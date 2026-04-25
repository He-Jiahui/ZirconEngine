use super::constants::WINDOW_MENU_INDEX;
use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::popup_layout::{popup_scroll_metrics, popup_viewport_extent};

impl HostMenuPointerBridge {
    pub(in crate::ui::slint_host::menu_pointer) fn clamp_popup_scroll_offset(&mut self) {
        let Some(menu_index) = self.state.open_menu_index else {
            self.state.popup_scroll_offset = 0.0;
            return;
        };
        if menu_index != WINDOW_MENU_INDEX {
            self.state.popup_scroll_offset = 0.0;
            return;
        }

        let (_, content_extent) = popup_scroll_metrics(&self.layout, menu_index);
        let viewport_extent = popup_viewport_extent(&self.layout, menu_index);
        let max_offset = (content_extent - viewport_extent).max(0.0);
        self.state.popup_scroll_offset = self.state.popup_scroll_offset.clamp(0.0, max_offset);
    }
}
