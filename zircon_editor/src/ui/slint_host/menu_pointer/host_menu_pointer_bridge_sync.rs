use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::host_menu_pointer_layout::HostMenuPointerLayout;
use super::host_menu_pointer_state::HostMenuPointerState;

impl HostMenuPointerBridge {
    pub(crate) fn sync(&mut self, layout: HostMenuPointerLayout, state: HostMenuPointerState) {
        self.layout = layout;
        self.state = state;
        self.clamp_popup_scroll_offset();
        self.rebuild_surface();
    }
}
