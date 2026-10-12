//! 范围滑块的最小值浮层只在控件够高、轨道够宽时显示，避免覆盖较窄普通滑块。

use super::super::super::super::data::FrameRect;
use super::super::metrics::workbench_slider_metrics;

/// 仅供范围下限值层使用；若高度或轨道宽度不足，返回 None 而保留滑块本体。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_range_min_value_rect(
    rect: &FrameRect,
    track_rect: &FrameRect,
) -> Option<FrameRect> {
    let metrics = workbench_slider_metrics();
    if rect.height < metrics.range_value_min_height || track_rect.width < metrics.value_width {
        return None;
    }
    Some(FrameRect {
        x: track_rect.x,
        y: track_rect.y + metrics.range_value_y_offset,
        width: metrics.value_width,
        height: metrics.range_value_height,
    })
}
