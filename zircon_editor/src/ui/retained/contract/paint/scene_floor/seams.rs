use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_style::surface_color;
use super::primitives::color_with_alpha_factor;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_floor_seam(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let color = surface_color(node);
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x - 1.0,
            y: rect.y,
            width: rect.width + 2.0,
            height: rect.height,
        },
        Some(clip.clone()),
        order,
        Some(color_with_alpha_factor(color, 0.38)),
        None,
        0.0,
        0.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
}
