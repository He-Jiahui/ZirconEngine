use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_inspector_row_geometry::{
    chevron_rect, field_rect, leading_affordance_rect, INSPECTOR_COUNT_WIDTH,
    INSPECTOR_FIELD_RIGHT_PAD, INSPECTOR_LABEL_WIDTH, INSPECTOR_ROW_TEXT_Y,
};
use super::super::template_inspector_row_glyphs::{
    push_inspector_cube_icon, push_inspector_down_chevron, push_inspector_swatch,
};
use super::super::template_inspector_row_kind::InspectorResourceKind;
use super::primitives::{push_field, push_label, push_text};
use super::style::{
    resource_chevron_size, resource_count_color, resource_glyph_color, resource_label_color,
    resource_value_color,
};

pub(super) fn push_resource_row(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    resource: InspectorResourceKind,
    opacity: f32,
) {
    let count_width = if resource == InspectorResourceKind::Material {
        INSPECTOR_COUNT_WIDTH
    } else {
        0.0
    };
    push_label(
        commands,
        rect,
        clip,
        order,
        node.text.trim(),
        resource_label_color(node),
        opacity,
    );
    if resource == InspectorResourceKind::Material {
        push_text(
            commands,
            FrameRect {
                x: rect.x + INSPECTOR_LABEL_WIDTH,
                y: rect.y + INSPECTOR_ROW_TEXT_Y,
                width: INSPECTOR_COUNT_WIDTH,
                height: (rect.height - INSPECTOR_ROW_TEXT_Y * 2.0).max(1.0),
            },
            clip,
            order + 1,
            "1",
            resource_count_color(node),
            opacity,
        );
    }

    let field = field_rect(
        rect,
        INSPECTOR_LABEL_WIDTH + count_width,
        rect.width - INSPECTOR_LABEL_WIDTH - count_width,
    );
    push_field(commands, node, &field, clip, order + 2, opacity);

    let leading = leading_affordance_rect(&field);
    let glyph_color = resource_glyph_color(node);
    match resource {
        InspectorResourceKind::Mesh => {
            push_inspector_cube_icon(commands, &leading, clip, order + 3, glyph_color, opacity)
        }
        InspectorResourceKind::Material => {
            push_inspector_swatch(commands, &leading, clip, order + 3, opacity)
        }
    }

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
        order + 4,
        node.value_text.trim(),
        resource_value_color(node),
        opacity,
    );
    let chevron_size = resource_chevron_size(node);
    push_inspector_down_chevron(
        commands,
        &chevron_rect(&field, chevron_size),
        clip,
        order + 5,
        glyph_color,
        opacity,
    );
}
