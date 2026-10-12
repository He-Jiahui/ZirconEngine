use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_viewport_scene_structure::push_base_surface;
use super::super::primitives::push_inset_rect;
use super::palette::{DOOR_INSET_LIGHT, DOOR_INSET_SHADOW};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_back_door(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_base_surface(commands, node, rect, clip, order, opacity);
    push_inset_rect(
        commands,
        rect,
        clip,
        order + 1,
        DOOR_INSET_LIGHT,
        8.0,
        opacity,
    );
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + rect.width * 0.5 - 1.0,
            y: rect.y + 8.0,
            width: 2.0,
            height: (rect.height - 16.0).max(1.0),
        },
        Some(clip.clone()),
        order + 2,
        Some(DOOR_INSET_SHADOW),
        None,
        0.0,
        0.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + 12.0,
            y: rect.y + rect.height * 0.50,
            width: (rect.width - 24.0).max(1.0),
            height: 2.0,
        },
        Some(clip.clone()),
        order + 3,
        Some(DOOR_INSET_SHADOW),
        None,
        0.0,
        0.0,
        opacity,
    ));
}
