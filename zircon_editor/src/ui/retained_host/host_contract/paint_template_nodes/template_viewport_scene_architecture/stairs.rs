use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_style::{border_color, surface_color, template_corner_radius};
use super::primitives::color_with_alpha_factor;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_side_stairs(
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
        Some(color_with_alpha_factor(surface_color(node), 0.26)),
        Some(color_with_alpha_factor(border_color(node), 0.72)),
        1.0,
        template_corner_radius(node),
        opacity,
    ));
    let step_color = color_with_alpha_factor(surface_color(node), 1.55);
    let mut y = rect.y + 10.0;
    let mut inset = 4.0;
    for step in 0..5 {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: rect.x + inset,
                y,
                width: (rect.width - inset - 10.0).max(1.0),
                height: 2.0,
            },
            Some(clip.clone()),
            order + 1 + step as i32,
            Some(step_color),
            None,
            0.0,
            0.0,
            opacity,
        ));
        y += 13.0;
        inset += 8.0;
    }
}
