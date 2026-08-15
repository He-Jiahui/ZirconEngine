use super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};
use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;

const OUTER_RADIUS_SCALE: f32 = 0.5;
const PLOT_RADIUS_SCALE: f32 = 0.25;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SampleGridMetrics {
    pub outer_radius: f32,
    pub plot_radius: f32,
    pub border_width: f32,
    pub grid_line_width: f32,
    pub selected_label_border_width: f32,
    pub selected_label_radius: f32,
}

pub(super) fn sample_grid_metrics() -> SampleGridMetrics {
    sample_grid_metrics_from_host(current_host_metrics())
}

pub(super) fn sample_grid_metrics_from_host(host: HostControlMetrics) -> SampleGridMetrics {
    SampleGridMetrics {
        outer_radius: host.radius_control * OUTER_RADIUS_SCALE,
        plot_radius: host.radius_control * PLOT_RADIUS_SCALE,
        border_width: host.border_width,
        grid_line_width: host.border_width,
        selected_label_border_width: host.border_width,
        selected_label_radius: host.radius_control * OUTER_RADIUS_SCALE,
    }
}

pub(super) const GRID_DASH_LENGTH: f32 = 3.0;
pub(super) const GRID_DASH_GAP: f32 = 4.0;
pub(super) const MIN_LEFT_GUTTER: f32 = 34.0;
pub(super) const MAX_LEFT_GUTTER: f32 = 48.0;
pub(super) const MIN_RIGHT_GUTTER: f32 = 10.0;
pub(super) const MAX_RIGHT_GUTTER: f32 = 18.0;
pub(super) const MIN_TOP_GUTTER: f32 = 40.0;
pub(super) const MAX_TOP_GUTTER: f32 = 48.0;
pub(super) const MIN_BOTTOM_GUTTER: f32 = 10.0;
pub(super) const MAX_BOTTOM_GUTTER: f32 = 18.0;
pub(super) const TICK_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE;
pub(super) const TICK_LINE_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;
pub(super) const AXIS_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_BODY_SIZE;
pub(super) const AXIS_LINE_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_BODY_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;
pub(super) const POINT_RADIUS: i32 = 5;
pub(super) const POINT_INTERIOR_RADIUS: i32 = 3;
pub(super) const POINT_EDGE_INSET: f32 = POINT_RADIUS as f32 + 1.0;
pub(super) const SAMPLE_LABEL_HEIGHT: f32 = 18.0;
pub(super) const SAMPLE_LABEL_MIN_WIDTH: f32 = 54.0;
pub(super) const SAMPLE_LABEL_POINT_GAP: f32 = 4.0;
pub(super) const AXIS_TITLE_EDGE_INSET: f32 = 3.0;
pub(super) const AXIS_TITLE_GAP: f32 = 4.0;
pub(super) const X_TICK_PLOT_GAP: f32 = 7.0;
