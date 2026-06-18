use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::PALETTE;
use super::super::render_commands::HostPaintCommand;
use super::super::template_inspector_row_geometry::{
    chevron_rect, nested_select_field_rect, shadow_check_rect, INSPECTOR_CHEVRON_SIZE,
    INSPECTOR_FIELD_RIGHT_PAD, INSPECTOR_FIELD_TEXT_X,
};
use super::super::template_inspector_row_glyphs::{
    push_inspector_check_tick, push_inspector_down_chevron,
};
use super::super::template_inspector_row_kind::{bool_display_value, bool_value};
use super::primitives::{push_field, push_nested_label, push_text};
use super::style::{resource_value_color, INSPECTOR_GLYPH_COLOR, RESOURCE_FIELD_BORDER};

pub(super) fn push_shadow_select_row(
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

pub(super) fn push_shadow_check_row(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_nested_label(commands, rect, clip, order, node.text.trim(), opacity);
    let check = shadow_check_rect(node, rect);
    let checked = bool_value(node.value_text.trim()) || node.checked || node.selected;
    commands.push(HostPaintCommand::quad(
        check.clone(),
        Some(clip.clone()),
        order + 1,
        Some(if checked {
            PALETTE.accent_soft
        } else {
            PALETTE.surface_inset
        }),
        Some(if checked {
            PALETTE.accent
        } else {
            RESOURCE_FIELD_BORDER
        }),
        1.0,
        3.0,
        opacity,
    ));
    if checked {
        push_inspector_check_tick(commands, &check, clip, order + 2, opacity);
    }
}
