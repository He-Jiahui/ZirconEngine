use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_viewport_scene_structure::push_base_surface;
use super::palette::{HANDRAIL_BOTTOM, HANDRAIL_POST};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_handrail(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_base_surface(commands, node, rect, clip, order, opacity);
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x,
            y: rect.y + rect.height + 1.0,
            width: rect.width,
            height: 2.0,
        },
        Some(clip.clone()),
        order + 1,
        Some(HANDRAIL_BOTTOM),
        None,
        0.0,
        0.0,
        opacity,
    ));
    for x in [rect.x + 36.0, rect.x + rect.width - 42.0] {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x,
                y: rect.y - 3.0,
                width: 4.0,
                height: 56.0,
            },
            Some(clip.clone()),
            order + 2,
            Some(HANDRAIL_POST),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}
