//! 行内提示的图标与文字预算；没有图标时回收左侧内容宽度，高提示为换行正文预留垂直带。

use super::super::super::super::data::FrameRect;
use super::common::fitted_centered_square;
use super::metrics::WorkbenchAlertMetrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_icon_rect(
    rect: &FrameRect,
    metrics: WorkbenchAlertMetrics,
) -> FrameRect {
    fitted_centered_square(rect, metrics.icon_left, metrics.icon_size)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_text_rect(
    rect: &FrameRect,
    icon: Option<&FrameRect>,
    metrics: WorkbenchAlertMetrics,
) -> Option<FrameRect> {
    let text_left = icon
        .map(|icon| icon.x + icon.width + metrics.text_gap)
        .unwrap_or(rect.x + metrics.icon_left);
    let text_right = rect.x + rect.width - metrics.text_right_inset;
    let multiline_height = (rect.height - metrics.text_vertical_inset * 2.0).max(0.0);
    let uses_multiline_band = multiline_height >= metrics.line_height * 2.0;
    (text_right > text_left).then(|| FrameRect {
        x: text_left,
        y: if uses_multiline_band {
            rect.y + metrics.text_vertical_inset
        } else {
            rect.y + (rect.height - metrics.line_height).max(0.0) * 0.5
        },
        width: text_right - text_left,
        height: if uses_multiline_band {
            multiline_height
        } else {
            metrics.line_height
        },
    })
}

#[cfg(test)]
#[path = "tests/inline.rs"]
mod tests;
