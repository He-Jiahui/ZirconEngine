//! 尾部标记必须完整位于可绘制行内并与clip相交；窄短行宁可省略标记。
//! 位置使用和文字列相同的宿主popup metrics，避免标题与标记占位不一致。

use super::super::super::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_geometry::intersect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn popup_row_adornment_rect(
    row_rect: &FrameRect,
    clip: &FrameRect,
) -> Option<FrameRect> {
    if !has_paintable_extent(row_rect) || !has_paintable_extent(clip) {
        return None;
    }
    let metrics = super::super::template_popup_rows::metrics::workbench_popup_row_metrics();
    let rect = FrameRect {
        x: row_rect.x + row_rect.width - metrics.adornment_right - metrics.adornment_size,
        y: row_rect.y + (row_rect.height - metrics.adornment_size).max(0.0) * 0.5,
        width: metrics.adornment_size,
        height: metrics.adornment_size,
    };
    (has_paintable_extent(&rect)
        && frame_is_within(row_rect, &rect)
        && intersect(clip, &rect).is_some())
    .then_some(rect)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn local_rect(
    origin: &FrameRect,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> FrameRect {
    FrameRect {
        x: origin.x + x,
        y: origin.y + y,
        width,
        height,
    }
}

fn has_paintable_extent(rect: &FrameRect) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

fn frame_is_within(container: &FrameRect, rect: &FrameRect) -> bool {
    rect.x >= container.x
        && rect.y >= container.y
        && rect.x + rect.width <= container.x + container.width
        && rect.y + rect.height <= container.y + container.height
}

#[cfg(test)]
#[path = "tests/geometry.rs"]
mod tests;
