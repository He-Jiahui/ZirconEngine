use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::metrics::workbench_slider_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_track_rect(
    rect: &FrameRect,
    value_rect: Option<&FrameRect>,
    has_label: bool,
    node: &TemplatePaneNodeData,
) -> FrameRect {
    let metrics = workbench_slider_metrics();
    let label_lane_width = if has_label {
        metrics.label_width + metrics.label_gap
    } else {
        0.0
    };
    let left = rect.x + label_lane_width + metrics.horizontal_inset + slider_track_offset_x(node);
    let right = (value_rect
        .map(|value| value.x - metrics.value_gap)
        .unwrap_or(rect.x + rect.width - metrics.horizontal_inset)
        + slider_track_width_delta(node))
    .max(left);
    FrameRect {
        x: left,
        y: rect.y + (rect.height - metrics.track_height).max(0.0) * 0.5,
        width: right - left,
        height: metrics.track_height,
    }
}

fn slider_track_offset_x(node: &TemplatePaneNodeData) -> f32 {
    node.layout_content_offset_x
}

fn slider_track_width_delta(node: &TemplatePaneNodeData) -> f32 {
    node.layout_first_cell_offset_x
}
