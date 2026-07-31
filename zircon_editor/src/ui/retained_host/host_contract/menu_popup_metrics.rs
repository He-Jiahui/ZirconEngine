pub(in crate::ui::retained_host::host_contract) use crate::ui::retained_host::menu_popup_contract::{
    MENU_POPUP_ROW_GAP, MENU_POPUP_ROW_HEIGHT,
};
use crate::ui::retained_host::menu_popup_contract::{
    MENU_POPUP_ANCHOR_GAP as SHARED_MENU_POPUP_ANCHOR_GAP, MENU_POPUP_EDGE_MARGIN,
    MENU_POPUP_MIN_HEIGHT, MENU_POPUP_PADDING,
};

use super::paint_text::measure_runtime_text_width;
use super::paint_theme::current_host_metrics;

pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_EDGE_INSET: f32 =
    MENU_POPUP_PADDING;
pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_ANCHOR_GAP: f32 =
    SHARED_MENU_POPUP_ANCHOR_GAP;
pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_SHELL_MARGIN: f32 =
    MENU_POPUP_EDGE_MARGIN;
pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_MIN_VISIBLE_HEIGHT: f32 =
    MENU_POPUP_MIN_HEIGHT;
pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_TEXT_INSET_X: f32 = 8.0;
pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_TEXT_INSET_Y: f32 = 6.0;

pub(crate) fn menu_popup_text_width(text: &str) -> f32 {
    let metrics = current_host_metrics();
    measure_runtime_text_width(text, metrics.font_body) + metrics.text_clip_guard
}

pub(in crate::ui::retained_host::host_contract) fn menu_popup_row_stride() -> f32 {
    MENU_POPUP_ROW_HEIGHT + MENU_POPUP_ROW_GAP
}

pub(in crate::ui::retained_host::host_contract) fn menu_popup_outer_padding() -> f32 {
    MENU_POPUP_EDGE_INSET * 2.0
}

pub(in crate::ui::retained_host::host_contract) fn menu_popup_shell_padding() -> f32 {
    MENU_POPUP_SHELL_MARGIN * 2.0
}
