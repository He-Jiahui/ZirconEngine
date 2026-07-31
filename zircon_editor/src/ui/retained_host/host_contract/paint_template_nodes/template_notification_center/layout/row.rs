use super::super::super::super::data::FrameRect;
use super::metrics::NotificationCenterMetrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn row_rect(
    panel_rect: &FrameRect,
    row: usize,
    metrics: &NotificationCenterMetrics,
) -> FrameRect {
    let horizontal_inset = metrics.row_inset_x.min(panel_rect.width.max(0.0) * 0.5);
    let panel_bottom = panel_rect.y + panel_rect.height.max(0.0);
    let y = (panel_rect.y + metrics.row_top + row as f32 * (metrics.row_height + metrics.row_gap))
        .min(panel_bottom);
    FrameRect {
        x: panel_rect.x + horizontal_inset,
        y,
        width: (panel_rect.width - horizontal_inset * 2.0).max(0.0),
        height: metrics.row_height.min((panel_bottom - y).max(0.0)),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn mark_rect(
    row_rect: &FrameRect,
    metrics: &NotificationCenterMetrics,
) -> FrameRect {
    let x = row_rect.x + metrics.mark_left.min(row_rect.width.max(0.0));
    let y = row_rect.y + metrics.mark_top.min(row_rect.height.max(0.0));
    FrameRect {
        x,
        y,
        width: metrics
            .mark_width
            .min((row_rect.x + row_rect.width - x).max(0.0)),
        height: metrics
            .mark_height
            .min((row_rect.y + row_rect.height - y).max(0.0)),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn title_rect(
    row_rect: &FrameRect,
    width: f32,
    metrics: &NotificationCenterMetrics,
) -> FrameRect {
    row_text_rect(
        row_rect,
        metrics.title_top,
        width,
        metrics.title_line_height,
        metrics,
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn message_rect(
    row_rect: &FrameRect,
    width: f32,
    metrics: &NotificationCenterMetrics,
) -> FrameRect {
    row_text_rect(
        row_rect,
        metrics.message_top,
        width,
        metrics.message_line_height,
        metrics,
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn row_text_width(
    row_rect: &FrameRect,
    metrics: &NotificationCenterMetrics,
) -> f32 {
    let left = row_text_left(row_rect, metrics);
    (row_rect.x + row_rect.width - metrics.text_right_inset - left).max(0.0)
}

fn row_text_left(row_rect: &FrameRect, metrics: &NotificationCenterMetrics) -> f32 {
    row_rect.x + metrics.text_left.min(row_rect.width.max(0.0))
}

fn row_text_rect(
    row_rect: &FrameRect,
    y_offset: f32,
    width: f32,
    height: f32,
    metrics: &NotificationCenterMetrics,
) -> FrameRect {
    let x = row_text_left(row_rect, metrics);
    let y = row_rect.y + y_offset.min(row_rect.height.max(0.0));
    FrameRect {
        x,
        y,
        width: width.min((row_rect.x + row_rect.width - x).max(0.0)),
        height: height.min((row_rect.y + row_rect.height - y).max(0.0)),
    }
}

#[cfg(test)]
mod tests {
    use super::super::metrics::notification_center_metrics;
    use super::*;

    #[test]
    fn rows_and_content_slots_stay_inside_a_narrow_short_panel() {
        let panel = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 18.0,
            height: 48.0,
        };
        let metrics = notification_center_metrics();

        for index in 0..=2 {
            let row = row_rect(&panel, index, &metrics);
            let width = row_text_width(&row, &metrics);

            assert_contained(row.clone(), &panel);
            assert_contained(mark_rect(&row, &metrics), &row);
            assert_contained(title_rect(&row, width, &metrics), &row);
            assert_contained(message_rect(&row, width, &metrics), &row);
        }
    }

    fn assert_contained(rect: FrameRect, parent: &FrameRect) {
        let epsilon = 0.000_1;
        assert!(rect.x >= parent.x - epsilon);
        assert!(rect.y >= parent.y - epsilon);
        assert!(rect.x + rect.width <= parent.x + parent.width + epsilon);
        assert!(rect.y + rect.height <= parent.y + parent.height + epsilon);
    }
}
