use super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};

const RADIUS_BORDER_MULTIPLIER: f32 = 2.0;
const CONTENT_TOP_ROW_MULTIPLIER: f32 = 2.0;
const ACTION_MIN_WIDTH_ROW_MULTIPLIER: f32 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchDialogMetrics
{
    pub padding_x: f32,
    pub title_top: f32,
    pub body_top: f32,
    pub content_gap: f32,
    pub content_action_gap: f32,
    pub title_font_size: f32,
    pub title_line_height: f32,
    pub body_font_size: f32,
    pub body_line_height: f32,
    pub severity_mark_width: f32,
    pub radius: f32,
    pub border_width: f32,
    pub action_bottom: f32,
    pub legacy_action_bottom: f32,
    pub action_gap: f32,
    pub action_stack_gap: f32,
    pub action_min_width: f32,
    pub action_height: f32,
    pub action_radius: f32,
    pub action_text_padding_x: f32,
    pub action_text_clip_guard: f32,
    pub action_font_size: f32,
    pub action_line_height: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_metrics(
) -> WorkbenchDialogMetrics {
    dialog_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_metrics_from_host(
    metrics: HostControlMetrics,
) -> WorkbenchDialogMetrics {
    // UE StandardDialog uses its normal font for headings/actions and small font for body copy.
    let title_font_size = metrics.font_body;
    let body_font_size = metrics.font_small;
    let action_font_size = metrics.font_body;
    WorkbenchDialogMetrics {
        padding_x: metrics.gap_l + metrics.gap_m,
        title_top: metrics.gap_l + metrics.gap_s + metrics.border_width * 2.0,
        body_top: metrics.row_height * CONTENT_TOP_ROW_MULTIPLIER,
        content_gap: metrics.gap_s + metrics.border_width,
        content_action_gap: metrics.gap_m + metrics.border_width,
        title_font_size,
        title_line_height: metrics.line_height(title_font_size),
        body_font_size,
        body_line_height: metrics.line_height(body_font_size),
        severity_mark_width: metrics.selection_indicator_width * 2.0,
        radius: metrics.radius_control + metrics.border_width * RADIUS_BORDER_MULTIPLIER,
        border_width: metrics.border_width,
        action_bottom: metrics.gap_m + metrics.border_width,
        legacy_action_bottom: metrics.gap_l + metrics.gap_m,
        action_gap: metrics.gap_l + metrics.gap_s,
        action_stack_gap: metrics.gap_s + metrics.border_width,
        action_min_width: metrics.row_height * ACTION_MIN_WIDTH_ROW_MULTIPLIER + metrics.gap_m,
        action_height: metrics.row_height,
        action_radius: metrics.radius_control,
        action_text_padding_x: metrics.gap_m + metrics.border_width * 2.0,
        action_text_clip_guard: metrics.text_clip_guard,
        action_font_size,
        action_line_height: metrics.line_height(action_font_size),
    }
}
