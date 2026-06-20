use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::style::{
    alert_background_color, alert_border_color, alert_border_width, alert_corner_radius,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_alert_surface(
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
        alert_background_color(node),
        alert_border_color(node),
        alert_border_width(node),
        alert_corner_radius(node, rect),
        opacity,
    ));
}
