use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::menu_popup_metrics::{
    menu_popup_shell_padding, MENU_POPUP_ANCHOR_GAP, MENU_POPUP_SHELL_MARGIN,
};

use super::content::{shell_content_height, shell_content_width};

pub(in crate::ui::retained_host::host_contract) fn constrained_submenu_popup_frame(
    presentation: &HostWindowPresentationData,
    anchor: &FrameRect,
    width: f32,
    requested_height: f32,
) -> FrameRect {
    let shell_width = shell_content_width(presentation);
    let shell_height = shell_content_height(presentation);
    let width = width
        .min((shell_width - menu_popup_shell_padding()).max(1.0))
        .max(1.0);
    let min_x = MENU_POPUP_SHELL_MARGIN;
    let max_x = (shell_width - width - MENU_POPUP_SHELL_MARGIN).max(min_x);
    let right_x = anchor.x + anchor.width + MENU_POPUP_ANCHOR_GAP;
    let left_x = anchor.x - width - MENU_POPUP_ANCHOR_GAP;
    let x = if right_x + width <= shell_width - MENU_POPUP_SHELL_MARGIN {
        right_x.clamp(min_x, max_x)
    } else {
        left_x.clamp(min_x, max_x)
    };
    let height = requested_height
        .min((shell_height - menu_popup_shell_padding()).max(1.0))
        .max(1.0);
    let min_y = MENU_POPUP_SHELL_MARGIN;
    let max_y = (shell_height - height - MENU_POPUP_SHELL_MARGIN).max(min_y);
    let y = anchor.y.clamp(min_y, max_y);
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
