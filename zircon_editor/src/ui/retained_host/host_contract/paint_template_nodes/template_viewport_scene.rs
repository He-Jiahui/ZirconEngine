mod commands;
mod geometry;
mod identity;

use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_viewport_scene_commands;
use identity::{ViewportSceneKind, viewport_scene_kind};

pub(in crate::ui::retained_host::host_contract) fn is_viewport_fallback_scene_node(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        viewport_scene_kind(node),
        Some(
            ViewportSceneKind::Container
                | ViewportSceneKind::SceneLayer
                | ViewportSceneKind::FloorGrate
                | ViewportSceneKind::FloorGrid
                | ViewportSceneKind::FloorPanel
                | ViewportSceneKind::FloorSeam
                | ViewportSceneKind::Backdrop
                | ViewportSceneKind::Ceiling
                | ViewportSceneKind::BackWall
                | ViewportSceneKind::FloorSurface
                | ViewportSceneKind::Cargo
                | ViewportSceneKind::PropBody
                | ViewportSceneKind::PropTop
                | ViewportSceneKind::CargoInner
                | ViewportSceneKind::Rack
                | ViewportSceneKind::Handrail
                | ViewportSceneKind::SidePanel
                | ViewportSceneKind::SideStairs
                | ViewportSceneKind::WallDetail
                | ViewportSceneKind::BackDoor
                | ViewportSceneKind::DoorCore
                | ViewportSceneKind::WallColumn
                | ViewportSceneKind::SoftLight
                | ViewportSceneKind::SoftShadow
                | ViewportSceneKind::FloorReflection
                | ViewportSceneKind::WallLight
                | ViewportSceneKind::Beacon
        )
    )
}

#[cfg(test)]
#[path = "template_viewport_scene_tests/mod.rs"]
mod tests;
