use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_inspector_row_geometry::{
    chevron_rect, field_rect, inspector_row_metrics, is_paintable_rect, leading_affordance_rect,
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
    let metrics = inspector_row_metrics();
    let field = field_rect(
        rect,
        metrics.label_width + count_width,
        rect.width - metrics.label_width - count_width,
    );
    push_field(commands, node, &field, clip, order, opacity);

    let leading = leading_affordance_rect(&field);
    let glyph_color = resource_glyph_color(node);
    if is_paintable_rect(&leading) {
        match resource {
            InspectorResourceKind::Mesh => {
                push_inspector_cube_icon(commands, &leading, clip, order + 1, glyph_color, opacity)
            }
            InspectorResourceKind::Material => {
                push_inspector_swatch(commands, &leading, clip, order + 1, opacity)
            }
        }
    }

    push_resource_value(commands, node, &field, &leading, clip, order + 2, opacity);
    let chevron = chevron_rect(&field, resource_chevron_size(node));
    if is_paintable_rect(&chevron) {
        push_inspector_down_chevron(commands, &chevron, clip, order + 3, glyph_color, opacity);
    }
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
    let metrics = inspector_row_metrics();
    let text_x = leading.x + leading.width + metrics.icon_text_gap;
    let inset_y = metrics.row_text_y.min((field.height.max(0.0)) * 0.5);
    push_text(
        commands,
        FrameRect {
            x: text_x,
            y: field.y + inset_y,
            width: (field.width - (text_x - field.x) - metrics.field_right_pad).max(0.0),
            height: (field.height.max(0.0) - inset_y * 2.0).max(0.0),
        },
        clip,
        order,
        node.value_text.trim(),
        resource_value_color(node),
        opacity,
    );
}
