use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_viewport_scene_structure::push_base_surface;
use super::super::primitives::push_expanded_layer;
use super::palette::{BEACON_CORE, BEACON_HALO};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_beacon(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_expanded_layer(
        commands,
        rect,
        clip,
        order,
        BEACON_HALO,
        8.0,
        4.0,
        8.0,
        opacity,
    );
    push_base_surface(commands, node, rect, clip, order + 1, opacity);
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + 2.0,
            y: rect.y + 4.0,
            width: (rect.width - 4.0).max(1.0),
            height: (rect.height - 8.0).max(1.0),
        },
        Some(clip.clone()),
        order + 2,
        Some(BEACON_CORE),
        None,
        0.0,
        1.0,
        opacity,
    ));
}
