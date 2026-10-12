use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_viewport_scene_structure::push_base_surface;
use super::colors::AXIS_GLOW;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_axis_origin(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x - 3.0,
            y: rect.y - 3.0,
            width: rect.width + 6.0,
            height: rect.height + 6.0,
        },
        Some(clip.clone()),
        order,
        Some(AXIS_GLOW),
        None,
        0.0,
        8.0,
        opacity,
    ));
    push_base_surface(commands, node, rect, clip, order + 1, opacity);
}
