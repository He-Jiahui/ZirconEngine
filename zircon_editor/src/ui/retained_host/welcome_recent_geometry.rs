use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

// Welcome panel width constraints are page-layout policy rather than control density.
const RECENT_PANEL_MIN_WIDTH: f32 = 220.0;
const RECENT_PANEL_MAX_WIDTH: f32 = 320.0;
const MAIN_PANEL_MIN_WIDTH: f32 = 280.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WelcomeRecentLayoutMetrics {
    outer_inset: f32,
    panel_top_inset: f32,
    header_height: f32,
    header_list_gap: f32,
    row_inset: f32,
    row_height: f32,
    row_gap: f32,
    row_text_inset: f32,
    row_action_inset: f32,
    row_action_gap: f32,
    row_action_height: f32,
    open_action_width: f32,
    remove_action_width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WelcomeRecentRowGeometry {
    pub row: UiFrame,
    pub text: UiFrame,
    pub open: UiFrame,
    pub remove: UiFrame,
}

pub(crate) fn current_welcome_recent_layout_metrics() -> WelcomeRecentLayoutMetrics {
    welcome_recent_layout_metrics_from_host(current_host_metrics())
}

pub(crate) fn welcome_recent_layout_metrics_from_host(
    metrics: HostControlMetrics,
) -> WelcomeRecentLayoutMetrics {
    let border_width = metrics.border_width.max(0.0);
    let outer_inset = (metrics.gap_l + metrics.gap_s + border_width * 2.0).max(0.0);
    let header_height = (metrics.control_large_height - border_width * 2.0)
        .max(metrics.control_default_height)
        .max(0.0);
    let row_height = (metrics.control_large_height + metrics.gap_m - border_width * 2.0)
        .max(metrics.control_default_height)
        .max(0.0);
    let row_action_height = (metrics.row_height - metrics.gap_s)
        .max(metrics.line_height(metrics.font_body))
        .max(0.0)
        .min(row_height);

    WelcomeRecentLayoutMetrics {
        outer_inset,
        panel_top_inset: outer_inset,
        header_height,
        header_list_gap: metrics.gap_m.max(0.0),
        row_inset: metrics.gap_m.max(0.0),
        row_height,
        row_gap: metrics.gap_m.max(0.0),
        row_text_inset: metrics.gap_l.max(0.0),
        row_action_inset: metrics.gap_m.max(0.0),
        row_action_gap: metrics.gap_s.max(0.0),
        row_action_height,
        open_action_width: (metrics.control_large_height + metrics.gap_s).max(0.0),
        remove_action_width: row_action_height,
    }
}

pub(crate) fn welcome_recent_viewport(pane_size: UiSize) -> UiFrame {
    welcome_recent_viewport_with_metrics(pane_size, current_welcome_recent_layout_metrics())
}

pub(crate) fn welcome_recent_viewport_with_metrics(
    pane_size: UiSize,
    metrics: WelcomeRecentLayoutMetrics,
) -> UiFrame {
    let outer_width = (pane_size.width - metrics.outer_inset * 2.0).max(0.0);
    let width_available_after_main = (outer_width - MAIN_PANEL_MIN_WIDTH).max(0.0);
    let recent_min = RECENT_PANEL_MIN_WIDTH.min(outer_width);
    let recent_max = RECENT_PANEL_MAX_WIDTH.min(outer_width);
    let recent_width = if recent_max >= recent_min {
        width_available_after_main.clamp(recent_min, recent_max)
    } else {
        recent_max
    };
    let y = metrics.outer_inset
        + metrics.panel_top_inset
        + metrics.header_height
        + metrics.header_list_gap;

    UiFrame::new(
        metrics.outer_inset,
        y,
        recent_width,
        (pane_size.height - y - metrics.outer_inset).max(0.0),
    )
}

pub(crate) fn welcome_recent_row_geometry(
    viewport: UiFrame,
    index: usize,
    scroll_offset: f32,
) -> WelcomeRecentRowGeometry {
    welcome_recent_row_geometry_with_metrics(
        viewport,
        index,
        scroll_offset,
        current_welcome_recent_layout_metrics(),
    )
}

pub(crate) fn welcome_recent_row_geometry_with_metrics(
    viewport: UiFrame,
    index: usize,
    scroll_offset: f32,
    metrics: WelcomeRecentLayoutMetrics,
) -> WelcomeRecentRowGeometry {
    let row = UiFrame::new(
        viewport.x + metrics.row_inset,
        viewport.y + metrics.row_inset + index as f32 * (metrics.row_height + metrics.row_gap)
            - scroll_offset.max(0.0),
        (viewport.width - metrics.row_inset * 2.0).max(0.0),
        metrics.row_height,
    );
    let action_y = row.y + (row.height - metrics.row_action_height).max(0.0) * 0.5;
    let remove_width = metrics
        .remove_action_width
        .min((row.width - metrics.row_action_inset * 2.0).max(0.0));
    let remove = UiFrame::new(
        (row.right() - metrics.row_action_inset - remove_width).max(row.x),
        action_y,
        remove_width,
        metrics.row_action_height.min(row.height.max(0.0)),
    );
    let open_width = metrics
        .open_action_width
        .min((remove.x - metrics.row_action_gap - row.x).max(0.0));
    let open = UiFrame::new(
        (remove.x - metrics.row_action_gap - open_width).max(row.x),
        action_y,
        open_width,
        metrics.row_action_height.min(row.height.max(0.0)),
    );
    let text_x = (row.x + metrics.row_text_inset).min(row.right());
    let text_right = (open.x - metrics.row_action_gap).clamp(text_x, row.right());
    let text = UiFrame::new(text_x, row.y, text_right - text_x, row.height);

    WelcomeRecentRowGeometry {
        row,
        text,
        open,
        remove,
    }
}

pub(crate) fn welcome_recent_content_height(item_count: usize) -> f32 {
    welcome_recent_content_height_with_metrics(item_count, current_welcome_recent_layout_metrics())
}

pub(crate) fn welcome_recent_content_height_with_metrics(
    item_count: usize,
    metrics: WelcomeRecentLayoutMetrics,
) -> f32 {
    if item_count == 0 {
        return 0.0;
    }
    metrics.row_inset * 2.0
        + item_count as f32 * metrics.row_height
        + (item_count.saturating_sub(1)) as f32 * metrics.row_gap
}

pub(crate) fn welcome_recent_visible_row_count(viewport_height: f32, item_count: usize) -> usize {
    welcome_recent_visible_row_count_with_metrics(
        viewport_height,
        item_count,
        current_welcome_recent_layout_metrics(),
    )
}

pub(crate) fn welcome_recent_visible_row_count_with_metrics(
    viewport_height: f32,
    item_count: usize,
    metrics: WelcomeRecentLayoutMetrics,
) -> usize {
    if item_count == 0 {
        return 0;
    }
    let inner_height = (viewport_height - metrics.row_inset * 2.0).max(0.0);
    if inner_height <= 0.0 {
        return 0;
    }
    ((inner_height / (metrics.row_height + metrics.row_gap)).ceil() as usize).min(item_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::METRICS;

    const EPSILON: f32 = 0.01;

    #[test]
    fn welcome_recent_geometry_keeps_compact_rows_and_actions_inside_responsive_columns() {
        let metrics = welcome_recent_layout_metrics_from_host(METRICS);
        assert_close(metrics.outer_inset, 18.0);
        assert_close(metrics.header_height, 46.0);
        assert_close(metrics.row_height, 54.0);
        assert_close(metrics.row_action_height, 24.0);
        assert_close(metrics.open_action_width, 52.0);
        for (pane_width, expected_recent_width) in [(560.0, 244.0), (640.0, 320.0), (900.0, 320.0)]
        {
            let viewport =
                welcome_recent_viewport_with_metrics(UiSize::new(pane_width, 520.0), metrics);
            assert_close(viewport.width, expected_recent_width);

            let first = welcome_recent_row_geometry_with_metrics(viewport, 0, 0.0, metrics);
            let second = welcome_recent_row_geometry_with_metrics(viewport, 1, 0.0, metrics);
            assert_close(first.row.height, 54.0);
            assert_close(second.row.y - first.row.y, 62.0);
            assert!(first.text.x >= first.row.x);
            assert!(first.text.right() <= first.open.x);
            assert!(first.open.right() <= first.remove.x);
            assert!(first.remove.right() <= first.row.right());
            for action in [first.open, first.remove] {
                assert!(action.y >= first.row.y);
                assert!(action.bottom() <= first.row.bottom());
            }
        }
    }

    #[test]
    fn welcome_recent_geometry_derives_content_and_visible_rows_from_one_metric_owner() {
        let metrics = welcome_recent_layout_metrics_from_host(METRICS);
        assert_close(welcome_recent_content_height_with_metrics(0, metrics), 0.0);
        assert_close(welcome_recent_content_height_with_metrics(1, metrics), 70.0);
        assert_close(
            welcome_recent_content_height_with_metrics(3, metrics),
            194.0,
        );
        assert_eq!(
            welcome_recent_visible_row_count_with_metrics(0.0, 8, metrics),
            0
        );
        assert_eq!(
            welcome_recent_visible_row_count_with_metrics(70.0, 8, metrics),
            1
        );
        assert_eq!(
            welcome_recent_visible_row_count_with_metrics(132.0, 8, metrics),
            2
        );
        assert_eq!(
            welcome_recent_visible_row_count_with_metrics(520.0, 2, metrics),
            2
        );
    }

    #[test]
    fn welcome_recent_geometry_reflows_from_compact_host_density() {
        let mut host = METRICS;
        host.control_default_height = 28.0;
        host.control_large_height = 40.0;
        host.row_height = 24.0;
        host.border_width = 1.5;
        host.gap_s = 3.0;
        host.gap_m = 6.0;
        host.gap_l = 9.0;
        host.font_body = 10.0;
        host.line_height_ratio = 1.2;
        let metrics = welcome_recent_layout_metrics_from_host(host);
        let viewport = welcome_recent_viewport_with_metrics(UiSize::new(560.0, 420.0), metrics);
        let row = welcome_recent_row_geometry_with_metrics(viewport, 1, 2.0, metrics);

        assert_close(metrics.outer_inset, 15.0);
        assert_close(metrics.header_height, 37.0);
        assert_close(metrics.row_height, 43.0);
        assert_close(metrics.row_action_height, 21.0);
        assert_close(metrics.open_action_width, 43.0);
        assert_close(row.row.y, viewport.y + 6.0 + 43.0 + 6.0 - 2.0);
        assert_close(
            welcome_recent_content_height_with_metrics(2, metrics),
            104.0,
        );
        assert!(row.text.right() <= row.open.x);
        assert!(row.open.right() <= row.remove.x);
    }

    #[test]
    fn welcome_recent_geometry_keeps_zero_width_actions_and_text_inside_narrow_rows() {
        let metrics = welcome_recent_layout_metrics_from_host(METRICS);
        let row = welcome_recent_row_geometry_with_metrics(
            UiFrame::new(0.0, 0.0, 12.0, 80.0),
            0,
            0.0,
            metrics,
        );

        assert_eq!(row.row.width, 0.0);
        assert_eq!(row.open.x, row.row.x);
        assert_eq!(row.remove.x, row.row.x);
        assert_eq!(row.text.x, row.row.x);
        assert_eq!(row.open.width, 0.0);
        assert_eq!(row.remove.width, 0.0);
        assert_eq!(row.text.width, 0.0);
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= EPSILON,
            "expected {expected}, got {actual}"
        );
    }
}
