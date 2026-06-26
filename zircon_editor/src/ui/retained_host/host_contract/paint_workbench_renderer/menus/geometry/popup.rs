use super::super::super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::super::super::menu_popup_metrics::{
    menu_popup_outer_padding, MENU_POPUP_ANCHOR_GAP, MENU_POPUP_MIN_VISIBLE_HEIGHT,
    MENU_POPUP_ROW_GAP, MENU_POPUP_ROW_HEIGHT, MENU_POPUP_SHELL_MARGIN,
};

use super::shell::{menu_shell_height, menu_shell_width};

pub(in crate::ui::retained_host::host_contract) fn menu_popup_height(item_count: usize) -> f32 {
    if item_count == 0 {
        0.0
    } else {
        menu_popup_outer_padding()
            + item_count as f32 * MENU_POPUP_ROW_HEIGHT
            + (item_count as f32 - 1.0) * MENU_POPUP_ROW_GAP
    }
}

pub(in crate::ui::retained_host::host_contract) fn constrained_menu_popup_frame(
    presentation: &HostWindowPresentationData,
    menu_frame: &FrameRect,
    width: f32,
    requested_height: f32,
) -> FrameRect {
    let shell_width = menu_shell_width(presentation);
    let shell_height = menu_shell_height(presentation);
    let width = width.min(shell_width).max(1.0);
    let popup_y = menu_frame.y + menu_frame.height + MENU_POPUP_ANCHOR_GAP;
    let x = menu_frame.x.clamp(0.0, (shell_width - width).max(0.0));
    let available_below = (shell_height - popup_y - MENU_POPUP_SHELL_MARGIN).max(0.0);
    let available_above = (menu_frame.y - MENU_POPUP_SHELL_MARGIN).max(0.0);
    let available_height = available_below
        .max(available_above)
        .max(MENU_POPUP_MIN_VISIBLE_HEIGHT)
        .min(shell_height);
    let height = requested_height.min(available_height).max(1.0);
    let y = if popup_y + height <= shell_height {
        popup_y
    } else {
        (menu_frame.y - height - MENU_POPUP_ANCHOR_GAP).max(0.0)
    };
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
