use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene_props::{
    push_cargo_detail, push_cargo_inner_frame, push_handrail, push_prop_body_detail,
    push_prop_top_detail, push_rack_detail,
};
use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene_structure::push_base_surface;

use super::super::identity::ViewportSceneKind;

pub(super) fn push_prop_scene_kind(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    kind: ViewportSceneKind,
) {
    match kind {
        ViewportSceneKind::Cargo => {
            push_base_surface(commands, node, rect, clip, order, opacity);
            push_cargo_detail(commands, rect, clip, order + 1, opacity);
        }
        ViewportSceneKind::PropBody => {
            push_prop_body_detail(commands, node, rect, clip, order, opacity);
        }
        ViewportSceneKind::PropTop => {
            push_prop_top_detail(commands, node, rect, clip, order, opacity);
        }
        ViewportSceneKind::CargoInner => {
            push_cargo_inner_frame(commands, rect, clip, order, opacity);
        }
        ViewportSceneKind::Rack => {
            push_base_surface(commands, node, rect, clip, order, opacity);
            push_rack_detail(commands, rect, clip, order + 1, opacity);
        }
        ViewportSceneKind::Handrail => {
            push_handrail(commands, node, rect, clip, order, opacity);
        }
        _ => {}
    }
}
