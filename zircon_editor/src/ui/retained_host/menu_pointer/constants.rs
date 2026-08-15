use zircon_runtime_interface::ui::event_ui::UiNodeId;

pub(in crate::ui::retained_host::menu_pointer) use crate::ui::retained_host::menu_popup_contract::WINDOW_MENU_INDEX;
pub(in crate::ui::retained_host::menu_pointer) use crate::ui::retained_host::menu_popup_contract::{
    MENU_POPUP_ANCHOR_GAP as POPUP_ANCHOR_GAP,
    MENU_POPUP_EDGE_MARGIN as POPUP_EDGE_MARGIN, MENU_POPUP_MIN_HEIGHT as POPUP_MIN_HEIGHT,
    MENU_POPUP_PADDING as POPUP_PADDING, MENU_POPUP_ROW_GAP as POPUP_ROW_GAP,
    MENU_POPUP_ROW_HEIGHT as POPUP_ROW_HEIGHT,
};

pub(in crate::ui::retained_host::menu_pointer) const ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
pub(in crate::ui::retained_host::menu_pointer) const DISMISS_NODE_ID: UiNodeId = UiNodeId::new(2);
pub(in crate::ui::retained_host::menu_pointer) const POPUP_NODE_ID: UiNodeId = UiNodeId::new(3);
pub(in crate::ui::retained_host::menu_pointer) const POPUP_NODE_ID_BASE: u64 = 3;
pub(in crate::ui::retained_host::menu_pointer) const MENU_BUTTON_NODE_ID_BASE: u64 = 10;
pub(in crate::ui::retained_host::menu_pointer) const MENU_ROUTE_ID_BASE: u64 = 54_000;

pub(in crate::ui::retained_host::menu_pointer) const POPUP_WIDTHS: [f32; 7] =
    [208.0, 186.0, 218.0, 172.0, 198.0, 224.0, 194.0];
