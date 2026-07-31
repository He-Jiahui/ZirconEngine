use super::super::super::super::data::FrameRect;
use super::super::metrics::WorkbenchPopupRowMetrics;

pub(super) fn popup_row_label_rect(
    row_rect: &FrameRect,
    metrics: &WorkbenchPopupRowMetrics,
    adornment_present: bool,
) -> FrameRect {
    let right_reserved = if adornment_present {
        metrics.adornment_reserved_width
    } else {
        metrics.text_right
    };
    FrameRect {
        x: row_rect.x + metrics.text_left,
        y: row_rect.y + metrics.text_top,
        width: (row_rect.width - metrics.text_left - right_reserved).max(0.0),
        height: popup_row_text_height(row_rect, metrics),
    }
}

pub(super) fn popup_row_shortcut_rect(
    row_rect: &FrameRect,
    metrics: &WorkbenchPopupRowMetrics,
) -> FrameRect {
    FrameRect {
        x: row_rect.x + row_rect.width * metrics.shortcut_left_ratio,
        y: row_rect.y + metrics.text_top,
        width: (row_rect.width * metrics.shortcut_width_ratio).max(0.0),
        height: popup_row_text_height(row_rect, metrics),
    }
}

fn popup_row_text_height(row_rect: &FrameRect, metrics: &WorkbenchPopupRowMetrics) -> f32 {
    (row_rect.height - metrics.text_top - metrics.text_bottom).max(0.0)
}
