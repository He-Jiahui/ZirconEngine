use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_style::{
    border_color, surface_color, template_border_width, template_corner_radius,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_base_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let border_width = template_border_width(node);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(surface_color(node)),
        (border_width > 0.0).then(|| border_color(node)),
        border_width,
        template_corner_radius(node),
        opacity,
    ));
}
