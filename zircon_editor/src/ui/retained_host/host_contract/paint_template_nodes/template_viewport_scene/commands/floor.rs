use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene_floor::{
    push_floor_grate_slots, push_floor_grid_line, push_floor_panel_detail, push_floor_seam,
};
use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene_structure::push_base_surface;

use super::super::identity::ViewportSceneKind;

pub(super) fn push_floor_scene_kind(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    kind: ViewportSceneKind,
) {
    match kind {
        ViewportSceneKind::FloorGrate => {
            push_base_surface(commands, node, rect, clip, order, opacity);
            push_floor_grate_slots(commands, rect, clip, order + 1, opacity);
        }
        ViewportSceneKind::FloorGrid => {
            push_floor_grid_line(commands, node, rect, clip, order, opacity);
        }
        ViewportSceneKind::FloorPanel => {
            push_floor_panel_detail(commands, node, rect, clip, order, opacity);
        }
        ViewportSceneKind::FloorSeam => {
            push_floor_seam(commands, node, rect, clip, order, opacity);
        }
        _ => {}
    }
}
