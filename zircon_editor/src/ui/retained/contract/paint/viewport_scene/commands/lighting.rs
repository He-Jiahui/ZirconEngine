use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene_light::{
    push_beacon, push_floor_reflection, push_soft_light, push_soft_shadow, push_wall_light,
};

use super::super::identity::ViewportSceneKind;

pub(super) fn push_lighting_scene_kind(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    kind: ViewportSceneKind,
) {
    match kind {
        ViewportSceneKind::SoftLight => {
            push_soft_light(commands, node, rect, clip, order, opacity);
        }
        ViewportSceneKind::SoftShadow => {
            push_soft_shadow(commands, node, rect, clip, order, opacity);
        }
        ViewportSceneKind::FloorReflection => {
            push_floor_reflection(commands, node, rect, clip, order, opacity);
        }
        ViewportSceneKind::WallLight => {
            push_wall_light(commands, node, rect, clip, order, opacity);
        }
        ViewportSceneKind::Beacon => {
            push_beacon(commands, node, rect, clip, order, opacity);
        }
        _ => {}
    }
}
