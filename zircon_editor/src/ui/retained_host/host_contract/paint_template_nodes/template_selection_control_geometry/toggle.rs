use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::{workbench_selection_control_metrics, WorkbenchSelectionControlMetrics};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toggle_track_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let metrics = workbench_selection_control_metrics();
    let track_width =
        toggle_track_width(node, metrics).min((rect.width - metrics.mark_inset_x * 2.0).max(0.0));
    let track_height = toggle_track_height(node, metrics).min(rect.height.max(0.0));
    FrameRect {
        x: (rect.x + rect.width - metrics.toggle_right_inset - track_width).max(rect.x),
        y: rect.y + (rect.height - track_height).max(0.0) * 0.5,
        width: track_width,
        height: track_height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toggle_thumb_rect(
    node: &TemplatePaneNodeData,
    track: &FrameRect,
) -> FrameRect {
    let metrics = workbench_selection_control_metrics();
    let thumb_size = toggle_thumb_size(node, metrics)
        .min(track.width)
        .min(track.height)
        .max(0.0);
    let available = (track.width - thumb_size - metrics.toggle_thumb_inset * 2.0).max(0.0);
    let offset = if node.checked || node.selected {
        available
    } else {
        0.0
    };
    FrameRect {
        x: track.x + metrics.toggle_thumb_inset + offset,
        y: track.y + (track.height - thumb_size).max(0.0) * 0.5,
        width: thumb_size,
        height: thumb_size,
    }
}

fn toggle_track_width(
    node: &TemplatePaneNodeData,
    metrics: WorkbenchSelectionControlMetrics,
) -> f32 {
    if node.value_number > 0.0 {
        node.value_number
    } else {
        metrics.toggle_track_width
    }
}

fn toggle_track_height(
    node: &TemplatePaneNodeData,
    metrics: WorkbenchSelectionControlMetrics,
) -> f32 {
    if node.layout_content_offset_y > 0.0 {
        node.layout_content_offset_y
    } else {
        metrics.toggle_track_height
    }
}

fn toggle_thumb_size(
    node: &TemplatePaneNodeData,
    metrics: WorkbenchSelectionControlMetrics,
) -> f32 {
    if node.layout_icon_size > 0.0 {
        node.layout_icon_size
    } else {
        metrics.toggle_thumb_size
    }
}
