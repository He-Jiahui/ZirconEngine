use super::workbench_menu_pointer_bridge::WorkbenchMenuPointerBridge;

impl WorkbenchMenuPointerBridge {
    pub(in crate::ui::slint_host::menu_pointer) fn close_popup(&mut self) {
        self.state.open_menu_index = None;
        self.state.hovered_menu_index = None;
        self.state.hovered_item_index = None;
        self.rebuild_surface();
    }

    pub(in crate::ui::slint_host::menu_pointer) fn open_popup(&mut self, menu_index: usize) {
        self.state.open_menu_index = Some(menu_index);
        self.state.hovered_menu_index = Some(menu_index);
        self.state.hovered_item_index = None;
        self.state.popup_scroll_offset = 0.0;
        self.rebuild_surface();
    }
}
