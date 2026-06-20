use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

use super::super::kind::ViewportSceneKind;
use super::architecture::architecture_scene_kind;
use super::floor::floor_scene_kind;
use super::gate::is_viewport_scene_candidate;
use super::gizmo::{center_gizmo_scene_kind, primary_gizmo_scene_kind};
use super::lighting::lighting_scene_kind;
use super::props::prop_scene_kind;
use super::surfaces::surface_scene_kind;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn viewport_scene_kind(
    node: &TemplatePaneNodeData,
) -> Option<ViewportSceneKind> {
    let id = node.control_id.as_str();
    if !is_viewport_scene_candidate(id) {
        return None;
    }

    lighting_scene_kind(id)
        .or_else(|| primary_gizmo_scene_kind(id))
        .or_else(|| surface_scene_kind(id))
        .or_else(|| floor_scene_kind(id))
        .or_else(|| prop_scene_kind(id))
        .or_else(|| architecture_scene_kind(id))
        .or_else(|| center_gizmo_scene_kind(id))
        .or(Some(ViewportSceneKind::SceneLayer))
}
