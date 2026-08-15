use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::menu_item_tree::menu_item_route_indices;
use super::menu_items_for_layout::menu_items_for_layout;

impl HostMenuPointerBridge {
    pub(super) fn refresh_popup_items(&mut self, force: bool) {
        let menu_index = self.state.open_menu_index;
        if !force && self.popup_menu_index == menu_index {
            return;
        }
        self.popup_menu_index = menu_index;
        self.popup_items = menu_index
            .map(|index| menu_items_for_layout(&self.layout, index).into_owned())
            .unwrap_or_default();
        self.popup_route_indices = menu_item_route_indices(&self.popup_items);
    }
}
