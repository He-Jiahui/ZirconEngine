use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_viewport_scene_structure::push_base_surface;
use super::palette::{PANEL_INSET_SHADOW, WARM_COLUMN_EDGE};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_wall_column(
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
            x: rect.x + 3.0,
            y: rect.y + 1.0,
            width: 3.0,
            height: (rect.height - 2.0).max(1.0),
        },
        Some(clip.clone()),
        order + 1,
        Some(WARM_COLUMN_EDGE),
        None,
        0.0,
        1.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + rect.width - 6.0,
            y: rect.y + 1.0,
            width: 3.0,
            height: (rect.height - 2.0).max(1.0),
        },
        Some(clip.clone()),
        order + 2,
        Some(PANEL_INSET_SHADOW),
        None,
        0.0,
        1.0,
        opacity,
    ));
}
