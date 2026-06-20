use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_inspector_row_geometry::{
    chevron_rect, field_rect, leading_affordance_rect, INSPECTOR_FIELD_RIGHT_PAD,
    INSPECTOR_LABEL_WIDTH,
};
use super::super::super::template_inspector_row_glyphs::{
    push_inspector_cube_icon, push_inspector_down_chevron, push_inspector_swatch,
};
use super::super::super::template_inspector_row_kind::InspectorResourceKind;
use super::super::primitives::{push_field, push_text};
use super::super::style::{resource_chevron_size, resource_glyph_color, resource_value_color};

pub(super) fn push_resource_field(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    count_width: f32,
    resource: InspectorResourceKind,
    opacity: f32,
) {
    let field = field_rect(
        rect,
        INSPECTOR_LABEL_WIDTH + count_width,
        rect.width - INSPECTOR_LABEL_WIDTH - count_width,
    );
    push_field(commands, node, &field, clip, order, opacity);

    let leading = leading_affordance_rect(&field);
    let glyph_color = resource_glyph_color(node);
    match resource {
        InspectorResourceKind::Mesh => {
            push_inspector_cube_icon(commands, &leading, clip, order + 1, glyph_color, opacity)
        }
        InspectorResourceKind::Material => {
            push_inspector_swatch(commands, &leading, clip, order + 1, opacity)
        }
    }

    push_resource_value(commands, node, &field, &leading, clip, order + 2, opacity);
    push_inspector_down_chevron(
        commands,
        &chevron_rect(&field, resource_chevron_size(node)),
        clip,
        order + 3,
        glyph_color,
        opacity,
    );
}

fn push_resource_value(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    field: &FrameRect,
    leading: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let text_x = leading.x + leading.width + 7.0;
    push_text(
        commands,
        FrameRect {
            x: text_x,
            y: field.y + 5.0,
            width: (field.x + field.width - text_x - INSPECTOR_FIELD_RIGHT_PAD).max(1.0),
            height: (field.height - 10.0).max(1.0),
        },
        clip,
        order,
        node.value_text.trim(),
        resource_value_color(node),
        opacity,
    );
}
