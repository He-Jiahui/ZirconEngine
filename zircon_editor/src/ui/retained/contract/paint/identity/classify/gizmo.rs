use super::super::kind::ViewportSceneKind;

pub(super) fn primary_gizmo_scene_kind(id: &str) -> Option<ViewportSceneKind> {
    if id.contains("Selection") {
        Some(ViewportSceneKind::SelectionEdge)
    } else if id == "WorkbenchViewportAxisOrigin" {
        Some(ViewportSceneKind::AxisOrigin)
    } else if id.as_bytes().windows(5).any(|window| {
        window[0] == b'A'
            && window[1] == b'x'
            && window[2] == b'i'
            && window[3] == b's'
            && matches!(window[4], b'X' | b'Y' | b'Z')
    }) {
        Some(ViewportSceneKind::AxisLine)
    } else {
        None
    }
}

#[cfg(test)]
#[path = "tests/gizmo.rs"]
mod tests;

pub(super) fn center_gizmo_scene_kind(id: &str) -> Option<ViewportSceneKind> {
    if id == "WorkbenchViewportGizmoCenter" {
        Some(ViewportSceneKind::GizmoCenter)
    } else {
        None
    }
}
