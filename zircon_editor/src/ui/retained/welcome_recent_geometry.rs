use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

use crate::ui::retained_host::host_contract::data::WelcomePaneLayoutData;
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
    safe_action_width: f32,
    recover_action_width: f32,
    remove_action_width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WelcomeRecentRowGeometry {
    pub row: UiFrame,
    pub text: UiFrame,
    pub open: UiFrame,
    pub safe: UiFrame,
    pub recover: UiFrame,
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
        safe_action_width: row_action_height,
        recover_action_width: row_action_height,
        remove_action_width: row_action_height,
    }
}

pub(crate) fn welcome_recent_viewport(pane_size: UiSize) -> UiFrame {
    welcome_recent_viewport_with_metrics(pane_size, current_welcome_recent_layout_metrics())
}

pub(crate) fn welcome_recent_viewport_for_layout(
    layout: &WelcomePaneLayoutData,
    pane_size: UiSize,
) -> UiFrame {
    match layout.recent_list_panel.as_ref().filter(|frame| {
        frame.x.is_finite()
            && frame.y.is_finite()
            && frame.width.is_finite()
            && frame.height.is_finite()
            && frame.width > 0.0
            && frame.height > 0.0
    }) {
        Some(frame) => UiFrame::new(frame.x, frame.y, frame.width, frame.height),
        None if layout.has_nodes => UiFrame::new(0.0, 0.0, 0.0, 0.0),
        None => welcome_recent_viewport(pane_size),
    }
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
    let recover_width = metrics
        .recover_action_width
        .min((remove.x - metrics.row_action_gap - row.x).max(0.0));
    let recover = UiFrame::new(
        (remove.x - metrics.row_action_gap - recover_width).max(row.x),
        action_y,
        recover_width,
        metrics.row_action_height.min(row.height.max(0.0)),
    );
    let safe_width = metrics
        .safe_action_width
        .min((recover.x - metrics.row_action_gap - row.x).max(0.0));
    let safe = UiFrame::new(
        (recover.x - metrics.row_action_gap - safe_width).max(row.x),
        action_y,
        safe_width,
        metrics.row_action_height.min(row.height.max(0.0)),
    );
    let open_width = metrics
        .open_action_width
        .min((safe.x - metrics.row_action_gap - row.x).max(0.0));
    let open = UiFrame::new(
        (safe.x - metrics.row_action_gap - open_width).max(row.x),
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
        safe,
        recover,
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
#[path = "tests/welcome_recent_geometry.rs"]
mod tests;
