//! 面板几何的共同约束：保留 DPI 后的小数位置，并让后续 painter 识别退化尺寸。
//! 这里不进行像素取整；命令入口负责决定尺寸过小时是否跳过整个面板。

use super::super::super::super::data::FrameRect;
use super::metrics::command_palette_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn min_frame_extent() -> f32 {
    command_palette_metrics().min_frame_extent
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paint_rect(
    rect: &FrameRect,
) -> FrameRect {
    let min_frame_extent = min_frame_extent();
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: rect.width.max(min_frame_extent),
        height: rect.height.max(min_frame_extent),
    }
}

#[cfg(test)]
#[path = "tests/common.rs"]
mod tests;

pub(super) fn symmetric_extent(inset: f32) -> f32 {
    inset * 2.0
}

pub(super) fn centered_offset(outer: f32, inner: f32) -> f32 {
    (outer - inner).max(0.0) * 0.5
}
