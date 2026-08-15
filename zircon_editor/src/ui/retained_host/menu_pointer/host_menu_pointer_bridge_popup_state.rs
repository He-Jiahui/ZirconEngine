use super::host_menu_pointer_bridge::HostMenuPointerBridge;

impl HostMenuPointerBridge {
    pub(in crate::ui::retained_host::menu_pointer) fn close_popup(&mut self) {
        if self.state.open_menu_index.is_none()
            && self.state.hovered_menu_index.is_none()
            && self.state.hovered_item_index.is_none()
            && self.state.hovered_item_path.is_empty()
            && self.state.open_submenu_path.is_empty()
        {
            return;
        }
        self.state.open_menu_index = None;
        self.state.hovered_menu_index = None;
        self.state.hovered_item_index = None;
        self.state.hovered_item_path.clear();
        self.state.open_submenu_path.clear();
        self.refresh_popup_items(false);
        self.rebuild_surface();
    }

    pub(in crate::ui::retained_host::menu_pointer) fn open_popup(&mut self, menu_index: usize) {
        self.state.open_menu_index = Some(menu_index);
        self.state.hovered_menu_index = Some(menu_index);
        self.state.hovered_item_index = None;
        self.state.hovered_item_path.clear();
        self.state.open_submenu_path.clear();
        self.state.popup_scroll_offset = 0.0;
        self.refresh_popup_items(false);
        self.rebuild_surface();
    }
}
