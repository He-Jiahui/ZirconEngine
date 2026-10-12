//! 动作标签与宽度预算共享 Runtime 字体测量，避免按字符数猜测按钮宽度。
//! 标签以投影数组索引提供，文本区域在按钮内部居中并受按钮可用宽度约束。

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::metrics::dialog_metrics;
use crate::ui::retained_host::host_contract::paint_text::measure_runtime_text_width;

pub(super) fn action_label(node: &TemplatePaneNodeData, index: usize) -> Option<String> {
    node.actions
        .get(index)
        .and_then(|action| non_empty(action.label.as_str()).map(str::to_string))
}

pub(super) fn action_width(text: &str) -> f32 {
    let metrics = dialog_metrics();
    (measure_runtime_text_width(text, metrics.action_font_size)
        + metrics.action_text_padding_x * 2.0
        + metrics.action_text_clip_guard)
        .max(metrics.action_min_width)
}

pub(super) fn action_text_frame(surface: &FrameRect, text: &str) -> FrameRect {
    let metrics = dialog_metrics();
    let available_width = (surface.width - metrics.action_text_padding_x * 2.0).max(0.0);
    let text_width = measure_runtime_text_width(text, metrics.action_font_size)
        .min(available_width)
        .max(0.0);
    FrameRect {
        x: surface.x + (surface.width - text_width).max(0.0) * 0.5,
        y: surface.y + (surface.height - metrics.action_line_height).max(0.0) * 0.5,
        width: text_width,
        height: metrics.action_line_height.min(surface.height).max(0.0),
    }
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

#[cfg(test)]
#[path = "tests/labels.rs"]
mod tests;
