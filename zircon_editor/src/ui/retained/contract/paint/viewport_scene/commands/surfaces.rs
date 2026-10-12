use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene_surfaces::{
    push_back_wall_surface, push_backdrop_surface, push_ceiling_surface, push_floor_surface,
};

use super::super::identity::ViewportSceneKind;

pub(super) fn push_surface_scene_kind(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    kind: ViewportSceneKind,
) {
    match kind {
        ViewportSceneKind::Backdrop => {
            push_backdrop_surface(commands, node, rect, clip, order, opacity);
        }
        ViewportSceneKind::Ceiling => {
            push_ceiling_surface(commands, node, rect, clip, order, opacity);
        }
        ViewportSceneKind::BackWall => {
            push_back_wall_surface(commands, node, rect, clip, order, opacity);
        }
        ViewportSceneKind::FloorSurface => {
            push_floor_surface(commands, node, rect, clip, order, opacity);
        }
        _ => {}
    }
}
