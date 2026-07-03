use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::{workbench_selection_control_metrics, WorkbenchSelectionControlMetrics};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn leading_mark_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let metrics = workbench_selection_control_metrics();
    let mark_size = selection_mark_size(node, metrics);
    FrameRect {
        x: rect.x + metrics.mark_inset_x,
        y: rect.y + (rect.height - mark_size).max(0.0) * 0.5,
        width: mark_size,
        height: mark_size,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn label_rect_after_mark(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    mark: &FrameRect,
) -> FrameRect {
    let metrics = workbench_selection_control_metrics();
    let x = mark.x + mark.width + selection_label_gap(node);
    FrameRect {
        x,
        y: rect.y + metrics.text_inset_y,
        width: (rect.x + rect.width - x - metrics.mark_inset_x).max(1.0),
        height: (rect.height - metrics.text_inset_y * 2.0).max(1.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selection_label_gap(
    node: &TemplatePaneNodeData,
) -> f32 {
    if node.layout_content_offset_x > 0.0 {
        node.layout_content_offset_x
    } else {
        workbench_selection_control_metrics().label_gap
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn centered_square(
    rect: &FrameRect,
    size: f32,
) -> FrameRect {
    let size = size.min(rect.width).min(rect.height).max(1.0);
    FrameRect {
        x: rect.x + (rect.width - size).max(0.0) * 0.5,
        y: rect.y + (rect.height - size).max(0.0) * 0.5,
        width: size,
        height: size,
    }
}

fn selection_mark_size(
    node: &TemplatePaneNodeData,
    metrics: WorkbenchSelectionControlMetrics,
) -> f32 {
    if node.layout_icon_size > 0.0 {
        node.layout_icon_size
    } else {
        metrics.mark_size
    }
}
