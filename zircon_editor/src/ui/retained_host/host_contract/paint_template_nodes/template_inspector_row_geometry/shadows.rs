use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::inspector_row_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn shadow_check_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let metrics = inspector_row_metrics();
    let row_width = finite_extent(rect.width);
    let row_height = finite_extent(rect.height);
    let left = (metrics.nested_label_width + shadow_check_content_offset_x(node)).min(row_width);
    let size = metrics
        .check_size
        .min((row_width - left).max(0.0))
        .min(row_height);
    FrameRect {
        x: finite_coordinate(rect.x) + left,
        y: finite_coordinate(rect.y) + (row_height - size) * 0.5,
        width: size,
        height: size,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn shadow_check_content_offset_x(
    node: &TemplatePaneNodeData,
) -> f32 {
    let declared_offset = node.layout_content_offset_x;
    if declared_offset.is_finite() && declared_offset > 0.0 {
        declared_offset
    } else {
        inspector_row_metrics().shadow_check_default_content_offset_x
    }
}

fn finite_extent(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_coordinate(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}
