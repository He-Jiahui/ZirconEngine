use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::style::{
    badge_root_background_color, badge_root_border_color, badge_root_border_width,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_badge_root_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let background = badge_root_background_color(node);
    let border_width = badge_root_border_width(node);
    let border = badge_root_border_color(node, border_width);
    if background.is_none() && border.is_none() && border_width <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        background,
        border,
        border_width.max(0.0),
        badge_root_corner_radius(node),
        opacity,
    ));
}

fn badge_root_corner_radius(node: &TemplatePaneNodeData) -> f32 {
    node.corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0)
}
