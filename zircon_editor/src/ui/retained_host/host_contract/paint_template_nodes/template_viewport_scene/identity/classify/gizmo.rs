use super::super::kind::ViewportSceneKind;

pub(super) fn primary_gizmo_scene_kind(id: &str) -> Option<ViewportSceneKind> {
    if id.contains("Selection") {
        Some(ViewportSceneKind::SelectionEdge)
    } else if id == "WorkbenchViewportAxisOrigin" {
        Some(ViewportSceneKind::AxisOrigin)
    } else if id.contains("AxisX") || id.contains("AxisY") || id.contains("AxisZ") {
        Some(ViewportSceneKind::AxisLine)
    } else {
        None
    }
}

pub(super) fn center_gizmo_scene_kind(id: &str) -> Option<ViewportSceneKind> {
    if id == "WorkbenchViewportGizmoCenter" {
        Some(ViewportSceneKind::GizmoCenter)
    } else {
        None
    }
}
