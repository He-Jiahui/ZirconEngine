use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::style::{
    configured_radius, field_fill_color, field_stroke_color, field_stroke_width,
    MUI_FIELD_FILLED_RADIUS, MUI_FIELD_OUTLINED_RADIUS,
};
use super::underline::push_underline;

pub(super) fn push_outlined_field(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        super::super::super::resolved_style_color(
            node.button_style.element.background_color.as_ref(),
        ),
        Some(field_stroke_color(node)),
        field_stroke_width(node),
        configured_radius(node, MUI_FIELD_OUTLINED_RADIUS),
        opacity,
    ));
}

pub(super) fn push_filled_field(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(field_fill_color(node)),
        None,
        0.0,
        configured_radius(node, MUI_FIELD_FILLED_RADIUS),
        opacity,
    ));
    push_underline(commands, node, rect, clip, order + 1, opacity);
}

pub(super) fn push_standard_field(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_underline(commands, node, rect, clip, order, opacity);
}
