//! 输入外观在节点槽内垂直居中，最大高度来自宿主密度；保留 DPI 后小数坐标供最终像素阶段对齐。

use super::super::super::data::FrameRect;
use super::metrics::axis_value_field_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_rect(
    rect: &FrameRect,
) -> FrameRect {
    let metrics = axis_value_field_metrics();
    let height = rect.height.min(metrics.max_height).max(0.0);
    FrameRect {
        x: rect.x,
        y: rect.y + (rect.height - height).max(0.0) * 0.5,
        width: rect.width.max(0.0),
        height,
    }
}

#[cfg(test)]
#[path = "tests/geometry.rs"]
mod tests;
