use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_style::{border_color, surface_color, template_corner_radius};
use super::super::primitives::color_with_alpha_factor;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_wall_detail_lines(
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
        Some(color_with_alpha_factor(surface_color(node), 0.16)),
        Some(color_with_alpha_factor(border_color(node), 0.52)),
        1.0,
        template_corner_radius(node),
        opacity,
    ));
    let line_color = color_with_alpha_factor(surface_color(node), 1.45);
    for (index, y_factor) in [0.20_f32, 0.38, 0.56, 0.74].into_iter().enumerate() {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: rect.x + rect.width * 0.12,
                y: rect.y + rect.height * y_factor,
                width: (rect.width * 0.76).max(1.0),
                height: 2.0,
            },
            Some(clip.clone()),
            order + 1 + index as i32,
            Some(line_color),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
    for x_factor in [0.24_f32, 0.50, 0.76] {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: rect.x + rect.width * x_factor,
                y: rect.y + rect.height * 0.10,
                width: 1.0,
                height: (rect.height * 0.78).max(1.0),
            },
            Some(clip.clone()),
            order + 6,
            Some(color_with_alpha_factor(line_color, 0.76)),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}
