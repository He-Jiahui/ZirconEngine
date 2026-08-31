use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_inspector_row_geometry::{
    chevron_rect, inspector_row_metrics, is_paintable_rect, nested_select_field_rect,
};
use crate::ui::retained_host::host_contract::paint_template_nodes::template_inspector_row_glyphs::push_inspector_down_chevron;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_inspector_row_kind::bool_display_value;

use super::super::primitives::{push_field, push_nested_label, push_text};
use super::super::style::{resource_glyph_color, resource_label_color, resource_value_color};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_shadow_select_row(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let metrics = inspector_row_metrics();
    push_nested_label(
        commands,
        rect,
        clip,
        order,
        node.text.trim(),
        resource_label_color(node),
        opacity,
    );
    let field = nested_select_field_rect(rect);
    push_field(commands, node, &field, clip, order + 1, opacity);
    let value = bool_display_value(&node.value_text);
    push_text(
        commands,
        FrameRect {
            x: field.x + metrics.field_text_x.min(field.width.max(0.0)),
            y: field.y + metrics.row_text_y.min(field.height.max(0.0) * 0.5),
            width: (field.width - metrics.field_text_x - metrics.field_right_pad).max(0.0),
            height: (field.height.max(0.0)
                - metrics.row_text_y.min(field.height.max(0.0) * 0.5) * 2.0)
                .max(0.0),
        },
        clip,
        order + 2,
        value,
        resource_value_color(node),
        opacity,
    );
    let chevron = chevron_rect(&field, metrics.chevron_size);
    if is_paintable_rect(&chevron) {
        push_inspector_down_chevron(
            commands,
            &chevron,
            clip,
            order + 3,
            resource_glyph_color(node),
            opacity,
        );
    }
}
