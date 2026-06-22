use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::content::{shell_content_height, shell_content_width};

pub(in crate::ui::retained_host::host_contract) fn constrained_submenu_popup_frame(
    presentation: &HostWindowPresentationData,
    anchor: &FrameRect,
    width: f32,
    requested_height: f32,
) -> FrameRect {
    let shell_width = shell_content_width(presentation);
    let shell_height = shell_content_height(presentation);
    let width = width.min((shell_width - 16.0).max(1.0)).max(1.0);
    let min_x = 8.0;
    let max_x = (shell_width - width - 8.0).max(min_x);
    let right_x = anchor.x + anchor.width + 3.0;
    let left_x = anchor.x - width - 3.0;
    let x = if right_x + width <= shell_width - 8.0 {
        right_x.clamp(min_x, max_x)
    } else {
        left_x.clamp(min_x, max_x)
    };
    let height = requested_height
        .min((shell_height - 16.0).max(1.0))
        .max(1.0);
    let min_y = 8.0;
    let max_y = (shell_height - height - 8.0).max(min_y);
    let y = anchor.y.clamp(min_y, max_y);
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
