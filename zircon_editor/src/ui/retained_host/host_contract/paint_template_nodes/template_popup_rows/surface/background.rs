use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::layers::popup_background_order;

mod style;

use style::popup_background_style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_popup_background(
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
    let style = popup_background_style(node);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        popup_background_order(order),
        Some(style.fill),
        Some(style.border),
        style.border_width,
        style.radius,
        opacity,
    ));
}
