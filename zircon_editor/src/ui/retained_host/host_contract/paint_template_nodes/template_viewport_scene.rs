use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_viewport_scene_architecture::{
    push_back_door, push_door_core, push_side_panel_detail, push_side_stairs, push_wall_column,
    push_wall_detail_lines,
};
use super::template_viewport_scene_floor::{
    push_floor_grate_slots, push_floor_grid_line, push_floor_panel_detail, push_floor_seam,
};
use super::template_viewport_scene_gizmos::{
    push_axis_line, push_axis_origin, push_gizmo_center, push_selection_glow,
};
use super::template_viewport_scene_light::{
    push_beacon, push_floor_reflection, push_soft_light, push_soft_shadow, push_wall_light,
};
use super::template_viewport_scene_props::{
    push_cargo_detail, push_cargo_inner_frame, push_handrail, push_prop_body_detail,
    push_prop_top_detail, push_rack_detail,
};
use super::template_viewport_scene_structure::push_base_surface;
use super::template_viewport_scene_surfaces::{
    push_back_wall_surface, push_backdrop_surface, push_ceiling_surface, push_floor_surface,
};

const VIEWPORT_CONTROL_PREFIX: &str = "WorkbenchViewport";

pub(super) fn push_viewport_scene_commands(
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
        ViewportSceneKind::SelectionEdge => {
            push_selection_glow(commands, &rect, clip, order, opacity);
            push_base_surface(commands, node, &rect, clip, order + 1, opacity);
        }
        ViewportSceneKind::AxisLine => {
            push_axis_line(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::AxisOrigin => {
            push_axis_origin(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::FloorGrate => {
            push_base_surface(commands, node, &rect, clip, order, opacity);
            push_floor_grate_slots(commands, &rect, clip, order + 1, opacity);
        }
        ViewportSceneKind::Backdrop => {
            push_backdrop_surface(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::Ceiling => {
            push_ceiling_surface(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::BackWall => {
            push_back_wall_surface(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::FloorSurface => {
            push_floor_surface(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::FloorGrid => {
            push_floor_grid_line(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::FloorPanel => {
            push_floor_panel_detail(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::FloorSeam => {
            push_floor_seam(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::Cargo => {
            push_base_surface(commands, node, &rect, clip, order, opacity);
            push_cargo_detail(commands, &rect, clip, order + 1, opacity);
        }
        ViewportSceneKind::PropBody => {
            push_prop_body_detail(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::PropTop => {
            push_prop_top_detail(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::CargoInner => {
            push_cargo_inner_frame(commands, &rect, clip, order, opacity);
        }
        ViewportSceneKind::Rack => {
            push_base_surface(commands, node, &rect, clip, order, opacity);
            push_rack_detail(commands, &rect, clip, order + 1, opacity);
        }
        ViewportSceneKind::SidePanel => {
            push_side_panel_detail(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::SideStairs => {
            push_side_stairs(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::WallDetail => {
            push_wall_detail_lines(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::BackDoor => {
            push_back_door(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::DoorCore => {
            push_door_core(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::WallColumn => {
            push_wall_column(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::Handrail => {
            push_handrail(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::SoftLight => {
            push_soft_light(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::SoftShadow => {
            push_soft_shadow(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::FloorReflection => {
            push_floor_reflection(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::WallLight => {
            push_wall_light(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::Beacon => {
            push_beacon(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::GizmoCenter => {
            push_gizmo_center(commands, &rect, clip, order, opacity);
        }
        ViewportSceneKind::SceneLayer => {
            push_base_surface(commands, node, &rect, clip, order, opacity);
        }
    }
    true
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ViewportSceneKind {
    Container,
    SceneLayer,
    Backdrop,
    Ceiling,
    BackWall,
    FloorSurface,
    FloorGrate,
    FloorGrid,
    FloorPanel,
    FloorSeam,
    Cargo,
    PropBody,
    PropTop,
    CargoInner,
    Rack,
    SidePanel,
    SideStairs,
    WallDetail,
    BackDoor,
    DoorCore,
    WallColumn,
    Handrail,
    SoftLight,
    SoftShadow,
    FloorReflection,
    WallLight,
    Beacon,
    SelectionEdge,
    AxisLine,
    AxisOrigin,
    GizmoCenter,
}

fn viewport_scene_kind(node: &TemplatePaneNodeData) -> Option<ViewportSceneKind> {
    let id = node.control_id.as_str();
    if !id.starts_with(VIEWPORT_CONTROL_PREFIX) || is_viewport_chrome_node(id) {
        return None;
    }

    if matches!(
        id,
        "WorkbenchViewportAxisXLabel"
            | "WorkbenchViewportAxisYLabel"
            | "WorkbenchViewportGizmoX"
            | "WorkbenchViewportGizmoY"
            | "WorkbenchViewportGizmoZ"
    ) {
        return None;
    }

    if id.contains("Lightwash") {
        Some(ViewportSceneKind::SoftLight)
    } else if id.contains("Shadow") {
        Some(ViewportSceneKind::SoftShadow)
    } else if id.contains("FloorReflection") {
        Some(ViewportSceneKind::FloorReflection)
    } else if id.contains("WallLight") {
        Some(ViewportSceneKind::WallLight)
    } else if id.contains("Beacon") {
        Some(ViewportSceneKind::Beacon)
    } else if id.contains("Selection") {
        Some(ViewportSceneKind::SelectionEdge)
    } else if id == "WorkbenchViewportAxisOrigin" {
        Some(ViewportSceneKind::AxisOrigin)
    } else if id.contains("AxisX") || id.contains("AxisY") || id.contains("AxisZ") {
        Some(ViewportSceneKind::AxisLine)
    } else if id == "WorkbenchViewportSurface" || id == "WorkbenchViewportGizmoPanel" {
        Some(ViewportSceneKind::Container)
    } else if id == "WorkbenchViewportBackdrop" {
        Some(ViewportSceneKind::Backdrop)
    } else if id == "WorkbenchViewportCeiling" {
        Some(ViewportSceneKind::Ceiling)
    } else if id == "WorkbenchViewportBackWall" {
        Some(ViewportSceneKind::BackWall)
    } else if id == "WorkbenchViewportFloor" {
        Some(ViewportSceneKind::FloorSurface)
    } else if id.contains("Grid") {
        Some(ViewportSceneKind::FloorGrid)
    } else if id.contains("FloorPanel") {
        Some(ViewportSceneKind::FloorPanel)
    } else if id.contains("FloorSeam") {
        Some(ViewportSceneKind::FloorSeam)
    } else if id.contains("FloorGrate") {
        Some(ViewportSceneKind::FloorGrate)
    } else if id == "WorkbenchViewportPropBody" {
        Some(ViewportSceneKind::PropBody)
    } else if id == "WorkbenchViewportPropTop" {
        Some(ViewportSceneKind::PropTop)
    } else if id.contains("Cargo") && id.contains("Inner") {
        Some(ViewportSceneKind::CargoInner)
    } else if id.contains("Cargo") {
        Some(ViewportSceneKind::Cargo)
    } else if id.contains("Rack") {
        Some(ViewportSceneKind::Rack)
    } else if id.contains("SideLeftStairs") {
        Some(ViewportSceneKind::SideStairs)
    } else if id == "WorkbenchViewportSideLeft" || id == "WorkbenchViewportSideRight" {
        Some(ViewportSceneKind::SidePanel)
    } else if id.contains("WallDetail") {
        Some(ViewportSceneKind::WallDetail)
    } else if id == "WorkbenchViewportBackDoor" {
        Some(ViewportSceneKind::BackDoor)
    } else if id == "WorkbenchViewportDoorCore" {
        Some(ViewportSceneKind::DoorCore)
    } else if id.contains("WallColumn") {
        Some(ViewportSceneKind::WallColumn)
    } else if id.contains("Handrail") {
        Some(ViewportSceneKind::Handrail)
    } else if id == "WorkbenchViewportGizmoCenter" {
        Some(ViewportSceneKind::GizmoCenter)
    } else {
        Some(ViewportSceneKind::SceneLayer)
    }
}

fn is_viewport_chrome_node(id: &str) -> bool {
    matches!(
        id,
        "WorkbenchViewportPanel"
            | "WorkbenchViewportToolbar"
            | "WorkbenchViewportToolbarFill"
            | "WorkbenchViewportMode"
            | "WorkbenchViewportLit"
            | "WorkbenchViewportAngle"
            | "WorkbenchViewportSpeed"
    )
}

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(0.0),
        height: rect.height.round().max(0.0),
    }
}

#[cfg(test)]
#[path = "template_viewport_scene_tests.rs"]
mod tests;
