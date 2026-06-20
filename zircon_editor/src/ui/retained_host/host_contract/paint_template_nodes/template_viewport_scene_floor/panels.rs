use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_viewport_scene_structure::push_base_surface;
use super::primitives::push_inset_rect;

const PANEL_INSET_LIGHT: [u8; 4] = [157, 178, 184, 26];
const PANEL_INSET_SHADOW: [u8; 4] = [0, 0, 0, 72];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_floor_panel_detail(
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
        PANEL_INSET_LIGHT,
        4.0,
        opacity,
    );
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + 1.0,
            y: rect.y + rect.height - 2.0,
            width: (rect.width - 2.0).max(1.0),
            height: 1.0,
        },
        Some(clip.clone()),
        order + 2,
        Some(PANEL_INSET_SHADOW),
        None,
        0.0,
        0.0,
        opacity,
    ));
}
