use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;

pub(super) fn push_data_grid_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    radius: f32,
    opacity: f32,
) {
    super::super::push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        super::super::node_background(node).unwrap_or(PALETTE.surface_inset),
        0.0,
        radius,
        opacity,
    );
}

pub(super) fn push_data_grid_header(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    radius: f32,
    header_height: f32,
    opacity: f32,
) {
    super::super::push_quad(
        commands,
        FrameRect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: header_height,
        },
        clip,
        order,
        PALETTE.surface_hover,
        0.0,
        radius,
        opacity,
    );
}
