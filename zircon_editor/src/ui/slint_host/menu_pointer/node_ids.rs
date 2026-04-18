use zircon_ui::UiNodeId;

use super::constants::{MENU_BUTTON_NODE_ID_BASE, MENU_ITEM_NODE_ID_BASE};

pub(in crate::ui::slint_host::menu_pointer) fn menu_button_node_id(index: usize) -> UiNodeId {
    UiNodeId::new(MENU_BUTTON_NODE_ID_BASE + index as u64)
}

pub(in crate::ui::slint_host::menu_pointer) fn menu_item_node_id(index: usize) -> UiNodeId {
    UiNodeId::new(MENU_ITEM_NODE_ID_BASE + index as u64)
}
