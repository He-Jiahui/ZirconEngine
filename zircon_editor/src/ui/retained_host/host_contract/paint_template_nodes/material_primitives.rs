use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_style_color::resolved_style_color;

mod alert;
mod avatar;
mod badge;
mod chip;
mod divider;
mod paper;
mod skeleton;
mod text_field;
mod timeline;

pub(super) fn push_material_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if alert::push_alert_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if chip::push_chip_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if avatar::push_avatar_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if badge::push_badge_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if skeleton::push_skeleton_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if paper::push_paper_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if timeline::push_timeline_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if divider::push_divider_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    false
}

pub(super) fn push_material_text_field_surface_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    text_field::push_text_field_surface_commands(commands, node, rect, clip, order, opacity)
}

pub(super) fn component_variant_contains(node: &TemplatePaneNodeData, expected: &str) -> bool {
    node.component_variant
        .as_str()
        .split(|character: char| {
            character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
        })
        .any(|part| part.eq_ignore_ascii_case(expected))
}

pub(super) fn first_non_empty<'a>(values: &[&'a str]) -> &'a str {
    values
        .iter()
        .copied()
        .find(|value| !value.is_empty())
        .unwrap_or_default()
}
