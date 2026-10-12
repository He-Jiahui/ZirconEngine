use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::geometry::chip_corner_radius;
use super::style::{chip_background_color, chip_border_color, chip_border_width};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chip_surface(
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
        chip_background_color(node),
        chip_border_color(node),
        chip_border_width(node),
        chip_corner_radius(node, rect),
        opacity,
    ));
}
