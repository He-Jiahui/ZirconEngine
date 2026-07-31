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
    icon: &FrameRect,
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
    let text_left = icon.x + icon.width + metrics.text_gap;
    (text_right > text_left).then(|| FrameRect {
        x: text_left,
        y: rect.y + (rect.height - metrics.line_height).max(0.0) * 0.5,
        width: text_right - text_left,
        height: metrics.line_height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::data::FrameRect;
    use crate::ui::retained_host::host_contract::paint_theme::METRICS;

    #[test]
    fn toast_action_uses_metric_derived_content_budget() {
        let metrics = super::super::metrics::toast_metrics_from_host(METRICS);
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: metrics.action_minimum_width - 0.1,
            height: METRICS.row_height,
        };
        assert!(!toast_has_action(&rect, metrics));

        let mut wide_rect = rect;
        wide_rect.width = metrics.action_minimum_width;
        assert!(toast_has_action(&wide_rect, metrics));
    }

    #[test]
    fn compact_toast_chrome_stays_inside_its_frame() {
        let metrics = super::super::metrics::toast_metrics_from_host(METRICS);
        let rect = FrameRect {
            x: 5.0,
            y: 7.0,
            width: 20.0,
            height: 10.0,
        };

        let icon = toast_icon_rect(&rect, metrics.icon_size, metrics);
        assert_eq!(icon.x, 17.0);
        assert_eq!(icon.y, 8.0);
        assert_eq!(icon.width, 8.0);
        assert_eq!(icon.height, 8.0);

        let close = toast_close_rect(&rect, metrics);
        assert_eq!(close.x, 5.0);
        assert_eq!(close.y, 7.0);
        assert_eq!(close.width, 10.0);
        assert_eq!(close.height, 10.0);
        assert!(!toast_has_action(&rect, metrics));
    }
}
