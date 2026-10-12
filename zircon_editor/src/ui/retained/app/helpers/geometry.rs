use super::super::*;
use crate::ui::retained_host::menu_popup_contract::{
    menu_popup_content_height, MENU_POPUP_ANCHOR_GAP, MENU_POPUP_EDGE_MARGIN, MENU_POPUP_MIN_HEIGHT,
};
use crate::ui::workbench::autolayout::ShellFrame;

pub(crate) fn viewport_size_from_frame(frame: ShellFrame) -> Option<UVec2> {
    let width = frame.width.max(0.0).round() as u32;
    let height = frame.height.max(0.0).round() as u32;
    if width == 0 || height == 0 {
        None
    } else {
        Some(UVec2::new(width, height))
    }
}

pub(crate) fn compute_window_menu_popup_height(
    shell_height: f32,
    button_frame: UiFrame,
    item_count: usize,
) -> f32 {
    let popup_y = button_frame.y + button_frame.height + MENU_POPUP_ANCHOR_GAP;
    let content_height = menu_popup_content_height(item_count).max(MENU_POPUP_MIN_HEIGHT);
    let available_height = (shell_height - popup_y - MENU_POPUP_EDGE_MARGIN)
        .max(MENU_POPUP_MIN_HEIGHT)
        .min(shell_height.max(1.0));
    content_height.min(available_height)
}

pub(crate) fn shell_region_group_key(region: ShellRegionId) -> &'static str {
    match region {
        ShellRegionId::Left => "left",
        ShellRegionId::Right => "right",
        ShellRegionId::Bottom => "bottom",
        ShellRegionId::Document => "document",
    }
}
