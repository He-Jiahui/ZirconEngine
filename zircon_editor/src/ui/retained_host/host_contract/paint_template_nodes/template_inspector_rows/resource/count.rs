use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_inspector_row_geometry::inspector_row_metrics;
use super::super::super::template_inspector_row_kind::InspectorResourceKind;
use super::super::primitives::push_text;
use super::super::style::resource_count_color;

pub(super) fn resource_count_width(resource: InspectorResourceKind) -> f32 {
    if resource == InspectorResourceKind::Material {
        inspector_row_metrics().count_width
    } else {
        0.0
    }
}

pub(super) fn push_resource_count(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    resource: InspectorResourceKind,
    opacity: f32,
) {
    if resource != InspectorResourceKind::Material {
        return;
    }
    let metrics = inspector_row_metrics();
    let row_width = finite_extent(rect.width);
    let row_height = finite_extent(rect.height);
    let left = metrics.label_width.min(row_width);
    let inset_y = metrics.row_text_y.min(row_height * 0.5);
    push_text(
        commands,
        FrameRect {
            x: finite_coordinate(rect.x) + left,
            y: finite_coordinate(rect.y) + inset_y,
            width: metrics.count_width.min((row_width - left).max(0.0)),
            height: (row_height - inset_y * 2.0).max(0.0),
        },
        clip,
        order,
        "1",
        resource_count_color(node),
        opacity,
    );
}

fn finite_extent(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_coordinate(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}
