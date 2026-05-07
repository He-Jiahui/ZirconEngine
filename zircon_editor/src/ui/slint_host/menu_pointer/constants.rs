use zircon_runtime_interface::ui::event_ui::UiNodeId;

pub(in crate::ui::slint_host::menu_pointer) const ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
pub(in crate::ui::slint_host::menu_pointer) const DISMISS_NODE_ID: UiNodeId = UiNodeId::new(2);
pub(in crate::ui::slint_host::menu_pointer) const POPUP_NODE_ID: UiNodeId = UiNodeId::new(3);
pub(in crate::ui::slint_host::menu_pointer) const POPUP_NODE_ID_BASE: u64 = 3;
pub(in crate::ui::slint_host::menu_pointer) const MENU_BUTTON_NODE_ID_BASE: u64 = 10;
pub(in crate::ui::slint_host::menu_pointer) const MENU_ITEM_NODE_ID_BASE: u64 = 100;
pub(in crate::ui::slint_host::menu_pointer) const MENU_ITEM_NODE_ID_LEVEL_STRIDE: u64 = 1000;

pub(in crate::ui::slint_host::menu_pointer) const POPUP_WIDTHS: [f32; 7] =
    [208.0, 186.0, 218.0, 172.0, 198.0, 224.0, 194.0];
pub(in crate::ui::slint_host::menu_pointer) const POPUP_PADDING: f32 = 6.0;
pub(in crate::ui::slint_host::menu_pointer) const POPUP_ROW_HEIGHT: f32 = 28.0;
pub(in crate::ui::slint_host::menu_pointer) const POPUP_ROW_GAP: f32 = 2.0;
pub(in crate::ui::slint_host::menu_pointer) const POPUP_ANCHOR_GAP: f32 = 3.0;
pub(in crate::ui::slint_host::menu_pointer) const POPUP_EDGE_MARGIN: f32 = 8.0;
pub(in crate::ui::slint_host::menu_pointer) const POPUP_MIN_HEIGHT: f32 = 72.0;
pub(in crate::ui::slint_host::menu_pointer) const WINDOW_MENU_INDEX: usize = 5;
