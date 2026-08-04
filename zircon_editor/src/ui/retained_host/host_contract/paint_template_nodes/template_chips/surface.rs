use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;
use super::metrics::{chip_border_width, chip_radius};
use super::style::{chip_border, chip_surface};
use crate::ui::retained_host::host_contract::paint_geometry::corner_radius_for_frame;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chip_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if intersect(rect, clip).is_none() {
        return;
    }
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(chip_surface(node)),
        Some(chip_border(node)),
        chip_border_width(),
        corner_radius_for_frame(rect, chip_radius()),
        opacity,
    ));
}
