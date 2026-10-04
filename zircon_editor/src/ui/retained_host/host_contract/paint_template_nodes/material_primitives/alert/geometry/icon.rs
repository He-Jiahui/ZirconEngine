use crate::ui::retained_host::host_contract::data::FrameRect;

use super::metrics::{
    alert_bounded_extent, ALERT_ICON_EDGE, ALERT_ICON_MARK_EDGE, ALERT_PADDING_X,
};

// 图标绘制消费此尺寸；图标及内部标记须受根框约束，避免极窄布局越界。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_icon_frame(
    rect: &FrameRect,
) -> FrameRect {
    let width = alert_bounded_extent(rect.width);
    let height = alert_bounded_extent(rect.height);
    let edge = ALERT_ICON_EDGE.min(width).min(height);
    let inset = ALERT_PADDING_X.min((width - edge).max(0.0));
    FrameRect {
        x: rect.x + inset,
        y: rect.y + (height - edge) * 0.5,
        width: edge,
        height: edge,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_icon_mark_frame(
    frame: &FrameRect,
) -> FrameRect {
    let edge = ALERT_ICON_MARK_EDGE
        .min(alert_bounded_extent(frame.width))
        .min(alert_bounded_extent(frame.height));
    FrameRect {
        x: frame.x + (alert_bounded_extent(frame.width) - edge) * 0.5,
        y: frame.y + (alert_bounded_extent(frame.height) - edge) * 0.5,
        width: edge,
        height: edge,
    }
}

#[cfg(test)]
#[path = "tests/icon.rs"]
mod tests;
