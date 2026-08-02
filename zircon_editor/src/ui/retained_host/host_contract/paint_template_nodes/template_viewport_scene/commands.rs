mod architecture;
mod floor;
mod gizmos;
mod lighting;
mod props;
mod surfaces;

use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene_structure::push_base_surface;

use super::geometry::pixel_aligned_rect;
use super::identity::{ViewportSceneKind, viewport_scene_kind};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_viewport_scene_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    let Some(kind) = viewport_scene_kind(node) else {
        return false;
    };

    let rect = pixel_aligned_rect(rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }

    match kind {
        ViewportSceneKind::Container => {}
        ViewportSceneKind::SceneLayer => {
            push_base_surface(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::SelectionEdge
        | ViewportSceneKind::AxisLine
        | ViewportSceneKind::AxisOrigin
        | ViewportSceneKind::GizmoCenter => {
            gizmos::push_gizmo_scene_kind(commands, node, &rect, clip, order, opacity, kind);
        }
        ViewportSceneKind::FloorGrate
        | ViewportSceneKind::FloorGrid
        | ViewportSceneKind::FloorPanel
        | ViewportSceneKind::FloorSeam => {
            floor::push_floor_scene_kind(commands, node, &rect, clip, order, opacity, kind);
        }
        ViewportSceneKind::Backdrop
        | ViewportSceneKind::Ceiling
        | ViewportSceneKind::BackWall
        | ViewportSceneKind::FloorSurface => {
            surfaces::push_surface_scene_kind(commands, node, &rect, clip, order, opacity, kind);
        }
        ViewportSceneKind::Cargo
        | ViewportSceneKind::PropBody
        | ViewportSceneKind::PropTop
        | ViewportSceneKind::CargoInner
        | ViewportSceneKind::Rack
        | ViewportSceneKind::Handrail => {
            props::push_prop_scene_kind(commands, node, &rect, clip, order, opacity, kind);
        }
        ViewportSceneKind::SidePanel
        | ViewportSceneKind::SideStairs
        | ViewportSceneKind::WallDetail
        | ViewportSceneKind::BackDoor
        | ViewportSceneKind::DoorCore
        | ViewportSceneKind::WallColumn => {
            architecture::push_architecture_scene_kind(
                commands, node, &rect, clip, order, opacity, kind,
            );
        }
        ViewportSceneKind::SoftLight
        | ViewportSceneKind::SoftShadow
        | ViewportSceneKind::FloorReflection
        | ViewportSceneKind::WallLight
        | ViewportSceneKind::Beacon => {
            lighting::push_lighting_scene_kind(commands, node, &rect, clip, order, opacity, kind);
        }
    }
    true
}
