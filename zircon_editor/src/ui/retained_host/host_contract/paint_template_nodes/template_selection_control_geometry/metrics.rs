use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const RADIO_DOT_SIZE: f32 =
    5.0;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TOGGLE_TRACK_WIDTH:
    f32 = 34.0;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TOGGLE_THUMB_SIZE: f32 =
    12.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchSelectionControlMetrics
{
    pub mark_inset_x: f32,
    pub mark_size: f32,
    pub border_width: f32,
    pub checkbox_radius: f32,
    pub label_gap: f32,
    pub text_inset_y: f32,
    pub radio_dot_size: f32,
    pub toggle_track_width: f32,
    pub toggle_track_height: f32,
    pub toggle_thumb_size: f32,
    pub toggle_right_inset: f32,
    pub toggle_thumb_inset: f32,
    pub font_size: f32,
    pub line_height: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_selection_control_metrics(
) -> WorkbenchSelectionControlMetrics {
    workbench_selection_control_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_selection_control_metrics_from_host(
    metrics: HostControlMetrics,
) -> WorkbenchSelectionControlMetrics {
    let font_size = metrics.font_body;
    let mark_inset_x = metrics.gap_m + metrics.border_width * 2.0;
    // Checkbox/radio marks use the same compact Icon16 density slot as panel
    // buttons; row height controls the hit row, not a larger authored mark.
    let mark_size = (metrics.row_height - metrics.gap_l).max(metrics.gap_m);
    let text_inset_y = metrics.gap_s + metrics.border_width;
    WorkbenchSelectionControlMetrics {
        mark_inset_x,
        mark_size,
        border_width: metrics.border_width,
        checkbox_radius: metrics.radius_control.min(mark_size * 0.5),
        label_gap: metrics.gap_m + metrics.border_width,
        text_inset_y,
        radio_dot_size: metrics.gap_s + metrics.border_width,
        toggle_track_width: metrics.gap_l * 2.0 + metrics.gap_m + metrics.border_width * 2.0,
        toggle_track_height: mark_size + metrics.border_width * 2.0,
        toggle_thumb_size: (mark_size - metrics.gap_s).max(0.0),
        toggle_right_inset: metrics.gap_m,
        toggle_thumb_inset: metrics.border_width * 2.0,
        font_size,
        line_height: metrics.line_height(font_size),
    }
}
