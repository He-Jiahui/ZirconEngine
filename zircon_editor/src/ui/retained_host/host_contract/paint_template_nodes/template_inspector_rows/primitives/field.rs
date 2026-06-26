use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::{METRICS, PALETTE};
use super::super::super::render_commands::HostPaintCommand;
use super::super::style::{resource_field_background, resource_field_border};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_field(
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
        Some(resource_field_background(node)),
        Some(if node.focused {
            PALETTE.focus_ring
        } else {
            resource_field_border(node)
        }),
        METRICS.border_width,
        METRICS.radius_control,
        opacity,
    ));
}
