pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_EDGE_INSET: f32 = 6.0;
pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_ROW_HEIGHT: f32 = 28.0;
pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_ROW_GAP: f32 = 2.0;
pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_ANCHOR_GAP: f32 = 3.0;
pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_SHELL_MARGIN: f32 = 8.0;
pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_MIN_VISIBLE_HEIGHT: f32 = 72.0;
pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_TEXT_INSET_X: f32 = 8.0;
pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_TEXT_INSET_Y: f32 = 6.0;
pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_SHORTCUT_RESERVED_WIDTH: f32 =
    34.0;

pub(in crate::ui::retained_host::host_contract) fn menu_popup_row_stride() -> f32 {
    MENU_POPUP_ROW_HEIGHT + MENU_POPUP_ROW_GAP
}

pub(in crate::ui::retained_host::host_contract) fn menu_popup_outer_padding() -> f32 {
    MENU_POPUP_EDGE_INSET * 2.0
}

pub(in crate::ui::retained_host::host_contract) fn menu_popup_shell_padding() -> f32 {
    MENU_POPUP_SHELL_MARGIN * 2.0
}
