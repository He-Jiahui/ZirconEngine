use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;
use super::super::{node_background, node_radius, push_quad};
use super::geometry::{picker_field_frame, picker_field_icon_frame};
use super::metrics::{PICKER_FIELD_RADIUS, PICKER_ROOT_BORDER_WIDTH, PICKER_SECONDARY};

pub(super) fn push_picker_field(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> FrameRect {
    push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        node_background(node).unwrap_or(PALETTE.surface_inset),
        PICKER_ROOT_BORDER_WIDTH,
        node_radius(node).max(PICKER_FIELD_RADIUS),
        opacity,
    );

    let field = picker_field_frame(rect);
    push_quad(
        commands,
        field.clone(),
        clip,
        order + 1,
        PALETTE.surface_inset,
        PICKER_ROOT_BORDER_WIDTH,
        PICKER_FIELD_RADIUS,
        opacity,
    );
    push_quad(
        commands,
        picker_field_icon_frame(&field),
        clip,
        order + 2,
        PICKER_SECONDARY,
        0.0,
        PICKER_FIELD_RADIUS,
        opacity,
    );
    field
}
