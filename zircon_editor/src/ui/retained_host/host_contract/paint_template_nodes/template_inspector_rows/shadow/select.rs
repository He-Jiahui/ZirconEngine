use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_inspector_row_geometry::{
    chevron_rect, nested_select_field_rect, INSPECTOR_CHEVRON_SIZE, INSPECTOR_FIELD_RIGHT_PAD,
    INSPECTOR_FIELD_TEXT_X,
};
use crate::ui::retained_host::host_contract::paint_template_nodes::template_inspector_row_glyphs::push_inspector_down_chevron;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_inspector_row_kind::bool_display_value;

use super::super::primitives::{push_field, push_nested_label, push_text};
use super::super::style::{resource_value_color, INSPECTOR_GLYPH_COLOR};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_shadow_select_row(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_nested_label(commands, rect, clip, order, node.text.trim(), opacity);
    let field = nested_select_field_rect(rect);
    push_field(commands, node, &field, clip, order + 1, opacity);
    let value = bool_display_value(node.value_text.trim());
    push_text(
        commands,
        FrameRect {
            x: field.x + INSPECTOR_FIELD_TEXT_X,
            y: field.y + 5.0,
            width: (field.width - INSPECTOR_FIELD_TEXT_X - INSPECTOR_FIELD_RIGHT_PAD).max(1.0),
            height: (field.height - 10.0).max(1.0),
        },
        clip,
        order + 2,
        value,
        resource_value_color(node),
        opacity,
    );
    push_inspector_down_chevron(
        commands,
        &chevron_rect(&field, INSPECTOR_CHEVRON_SIZE),
        clip,
        order + 3,
        INSPECTOR_GLYPH_COLOR,
        opacity,
    );
}
