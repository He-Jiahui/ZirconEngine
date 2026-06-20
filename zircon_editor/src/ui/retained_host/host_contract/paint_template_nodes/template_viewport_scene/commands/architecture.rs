use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene_architecture::{
    push_back_door, push_door_core, push_side_panel_detail, push_side_stairs, push_wall_column,
    push_wall_detail_lines,
};

use super::super::identity::ViewportSceneKind;

pub(super) fn push_architecture_scene_kind(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    kind: ViewportSceneKind,
) {
    match kind {
        ViewportSceneKind::SidePanel => {
            push_side_panel_detail(commands, node, rect, clip, order, opacity);
        }
        ViewportSceneKind::SideStairs => {
            push_side_stairs(commands, node, rect, clip, order, opacity);
        }
        ViewportSceneKind::WallDetail => {
            push_wall_detail_lines(commands, node, rect, clip, order, opacity);
        }
        ViewportSceneKind::BackDoor => {
            push_back_door(commands, node, rect, clip, order, opacity);
        }
        ViewportSceneKind::DoorCore => {
            push_door_core(commands, node, rect, clip, order, opacity);
        }
        ViewportSceneKind::WallColumn => {
            push_wall_column(commands, node, rect, clip, order, opacity);
        }
        _ => {}
    }
}
