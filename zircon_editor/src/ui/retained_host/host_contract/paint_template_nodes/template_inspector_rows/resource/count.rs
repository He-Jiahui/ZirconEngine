use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_inspector_row_geometry::{
    INSPECTOR_COUNT_WIDTH, INSPECTOR_LABEL_WIDTH, INSPECTOR_ROW_TEXT_Y,
};
use super::super::super::template_inspector_row_kind::InspectorResourceKind;
use super::super::primitives::push_text;
use super::super::style::resource_count_color;

pub(super) fn resource_count_width(resource: InspectorResourceKind) -> f32 {
    if resource == InspectorResourceKind::Material {
        INSPECTOR_COUNT_WIDTH
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
    push_text(
        commands,
        FrameRect {
            x: rect.x + INSPECTOR_LABEL_WIDTH,
            y: rect.y + INSPECTOR_ROW_TEXT_Y,
            width: INSPECTOR_COUNT_WIDTH,
            height: (rect.height - INSPECTOR_ROW_TEXT_Y * 2.0).max(1.0),
        },
        clip,
        order,
        "1",
        resource_count_color(node),
        opacity,
    );
}
