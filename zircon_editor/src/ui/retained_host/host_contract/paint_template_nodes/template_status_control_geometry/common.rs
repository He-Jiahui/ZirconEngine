//! 状态栏偏移是视觉位置声明；产生命令的调用方仍须决定是否允许内容越过原节点矩形。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn has_paintable_status_control_extent(
    rect: &FrameRect,
) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn frame_is_within(
    outer: &FrameRect,
    inner: &FrameRect,
) -> bool {
    has_paintable_status_control_extent(outer)
        && has_paintable_status_control_extent(inner)
        && inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}

/// 为状态 chip/图标应用视觉偏移，保持原尺寸；返回矩形不保证仍被原矩形包含。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_control_offset_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x + node.layout_offset_x,
        y: rect.y + node.layout_offset_y,
        width: rect.width,
        height: rect.height,
    }
}
