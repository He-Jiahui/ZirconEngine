//! 轴文字在节点槽中垂直居中并收紧到可用高度；无效和退化尺寸不应让后续 text 命令扩展槽位。

use super::super::super::super::data::FrameRect;
use super::super::metrics::AxisLabelMetrics;

pub(super) fn axis_label_text_rect(rect: &FrameRect, metrics: &AxisLabelMetrics) -> FrameRect {
    let x = finite_coordinate(rect.x);
    let y = finite_coordinate(rect.y);
    let width = finite_non_negative(rect.width);
    let height = finite_non_negative(rect.height);
    let line_height = finite_non_negative(metrics.line_height).min(height);
    FrameRect {
        x,
        y: y + finite_non_negative(height - line_height) * 0.5,
        width,
        height: line_height,
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_coordinate(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

#[cfg(test)]
#[path = "tests/geometry.rs"]
mod tests;
