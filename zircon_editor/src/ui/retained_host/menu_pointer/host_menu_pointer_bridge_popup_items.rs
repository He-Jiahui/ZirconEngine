use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::menu_item_tree::menu_item_route_indices;
use super::menu_items_for_layout::menu_items_for_layout;

impl HostMenuPointerBridge {
    pub(super) fn refresh_popup_items(&mut self) {
        let menu_index = self.state.open_menu_index;
        let Some(menu_index) = menu_index else {
            if self.popup_menu_index.is_none()
                && self.popup_items.is_empty()
                && self.popup_route_indices.is_empty()
            {
                return;
            }
            self.popup_menu_index = None;
            self.popup_items.clear();
            self.popup_route_indices.clear();
            return;
        };

        let next_items = menu_items_for_layout(&self.layout, menu_index);
        if self.popup_menu_index == Some(menu_index)
            && self.popup_items.as_slice() == next_items.as_ref()
        {
            return;
        }

        self.popup_menu_index = Some(menu_index);
        self.popup_items = next_items.into_owned();
        self.popup_route_indices = menu_item_route_indices(&self.popup_items);
    }
}
