use super::super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::super::render_commands::HostPaintCommand;
use super::super::super::geometry::badge_overlay_radius;
use super::super::super::style::{
    badge_overlay_background_color, badge_overlay_border_color, badge_overlay_border_width,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_badge_overlay_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    dot: bool,
    opacity: f32,
) {
    let background = badge_overlay_background_color(node);
    commands.push(HostPaintCommand::quad(
        rect,
        Some(clip.clone()),
        order,
        Some(background),
        Some(badge_overlay_border_color(node, background)),
        badge_overlay_border_width(node),
        badge_overlay_radius(dot),
        opacity,
    ));
}
