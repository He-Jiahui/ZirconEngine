use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::host_menu_pointer_layout::HostMenuPointerLayout;
use super::host_menu_pointer_state::HostMenuPointerState;

impl HostMenuPointerBridge {
    pub(crate) fn sync(
        &mut self,
        layout: HostMenuPointerLayout,
        state: HostMenuPointerState,
    ) -> bool {
        let layout_changed = self.layout != layout;
        if !layout_changed && self.state == state {
            return false;
        }

        self.layout = layout;
        self.state = state;
        self.clamp_menu_bar_scroll_offset();
        self.refresh_popup_items(layout_changed);
        self.clamp_popup_scroll_offset();
        self.rebuild_surface();
        true
    }
}
