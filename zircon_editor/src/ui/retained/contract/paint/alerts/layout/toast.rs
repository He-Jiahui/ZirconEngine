//! toast 的图标、消息与尾部反馈共享内容预算；has_action 是几何容量判断，不代表上游存在可执行动作。

use super::super::super::super::data::FrameRect;
use super::common::fitted_centered_square;
use super::metrics::WorkbenchToastMetrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toast_icon_rect(
    rect: &FrameRect,
    icon_size: f32,
    metrics: WorkbenchToastMetrics,
) -> FrameRect {
    fitted_centered_square(rect, metrics.icon_left, icon_size)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toast_close_rect(
    rect: &FrameRect,
    metrics: WorkbenchToastMetrics,
) -> FrameRect {
    let close_right = (rect.x + rect.width - metrics.trailing_inset).max(rect.x);
    let close_size = metrics
        .close_size
        .min(rect.height.max(0.0))
        .min((close_right - rect.x).max(0.0));
    FrameRect {
        x: close_right - close_size,
        y: rect.y + (rect.height - close_size).max(0.0) * 0.5,
        width: close_size,
        height: close_size,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toast_has_action(
    rect: &FrameRect,
    metrics: WorkbenchToastMetrics,
) -> bool {
    rect.width >= metrics.action_minimum_width
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toast_action_rect(
    rect: &FrameRect,
    close: &FrameRect,
    metrics: WorkbenchToastMetrics,
) -> FrameRect {
    FrameRect {
        x: close.x - metrics.action_width,
        y: rect.y + (rect.height - metrics.line_height).max(0.0) * 0.5,
        width: metrics.action_width,
        height: metrics.line_height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toast_text_rect(
    rect: &FrameRect,
    icon: Option<&FrameRect>,
    close: &FrameRect,
    has_action: bool,
    metrics: WorkbenchToastMetrics,
) -> Option<FrameRect> {
    let action_left = close.x - metrics.action_width;
    let text_right = if has_action {
        action_left - metrics.action_gap
    } else {
        rect.x + rect.width - metrics.trailing_inset
    };
    let text_left = icon
        .map(|icon| icon.x + icon.width + metrics.text_gap)
        .unwrap_or(rect.x + metrics.icon_left);
    (text_right > text_left).then(|| FrameRect {
        x: text_left,
        y: rect.y + (rect.height - metrics.line_height).max(0.0) * 0.5,
        width: text_right - text_left,
        height: metrics.line_height,
    })
}

#[cfg(test)]
#[path = "tests/toast.rs"]
mod tests;
