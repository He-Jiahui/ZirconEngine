use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::style::chat_surface_color;

pub(super) fn push_agent_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = super::super::super::node_radius(node).max(8.0);
    super::super::super::push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        super::super::super::node_background(node).unwrap_or_else(|| chat_surface_color(node)),
        0.0,
        radius,
        opacity,
    );
}
