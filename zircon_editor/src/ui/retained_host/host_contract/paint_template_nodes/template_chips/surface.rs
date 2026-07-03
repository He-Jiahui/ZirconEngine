use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::metrics::{chip_border_width, chip_radius};
use super::style::{chip_border, chip_surface};

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
        Some(chip_surface(node)),
        Some(chip_border(node)),
        chip_border_width(),
        chip_radius(),
        opacity,
    ));
}
