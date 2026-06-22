use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::content::{shell_content_height, shell_content_width};

pub(in crate::ui::retained_host::host_contract) fn constrained_menu_popup_frame(
    presentation: &HostWindowPresentationData,
    menu_frame: &FrameRect,
    width: f32,
    requested_height: f32,
) -> FrameRect {
    let shell_width = shell_content_width(presentation);
    let shell_height = shell_content_height(presentation);
    let width = width.min(shell_width).max(1.0);
    let popup_y = menu_frame.y + menu_frame.height + 3.0;
    let x = menu_frame.x.clamp(0.0, (shell_width - width).max(0.0));
    let available_below = (shell_height - popup_y - 8.0).max(0.0);
    let available_above = (menu_frame.y - 8.0).max(0.0);
    let available_height = available_below
        .max(available_above)
        .max(72.0)
        .min(shell_height);
    let height = requested_height.min(available_height).max(1.0);
    let y = if popup_y + height <= shell_height {
        popup_y
    } else {
        (menu_frame.y - height - 3.0).max(0.0)
    };
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
