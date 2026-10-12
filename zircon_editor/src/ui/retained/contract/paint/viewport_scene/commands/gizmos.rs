use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene_gizmos::{
    push_axis_line, push_axis_origin, push_gizmo_center, push_selection_glow,
};
use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene_structure::push_base_surface;

use super::super::identity::ViewportSceneKind;

pub(super) fn push_gizmo_scene_kind(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    kind: ViewportSceneKind,
) {
    match kind {
        ViewportSceneKind::SelectionEdge => {
            push_selection_glow(commands, rect, clip, order, opacity);
            push_base_surface(commands, node, rect, clip, order + 1, opacity);
        }
        ViewportSceneKind::AxisLine => {
            push_axis_line(commands, node, rect, clip, order, opacity);
        }
        ViewportSceneKind::AxisOrigin => {
            push_axis_origin(commands, node, rect, clip, order, opacity);
        }
        ViewportSceneKind::GizmoCenter => {
            push_gizmo_center(commands, rect, clip, order, opacity);
        }
        _ => {}
    }
}
