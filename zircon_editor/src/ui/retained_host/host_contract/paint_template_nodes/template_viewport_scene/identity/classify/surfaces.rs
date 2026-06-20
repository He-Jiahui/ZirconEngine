use super::super::kind::ViewportSceneKind;

pub(super) fn surface_scene_kind(id: &str) -> Option<ViewportSceneKind> {
    if id == "WorkbenchViewportSurface" || id == "WorkbenchViewportGizmoPanel" {
        Some(ViewportSceneKind::Container)
    } else if id == "WorkbenchViewportBackdrop" {
        Some(ViewportSceneKind::Backdrop)
    } else if id == "WorkbenchViewportCeiling" {
        Some(ViewportSceneKind::Ceiling)
    } else if id == "WorkbenchViewportBackWall" {
        Some(ViewportSceneKind::BackWall)
    } else if id == "WorkbenchViewportFloor" {
        Some(ViewportSceneKind::FloorSurface)
    } else {
        None
    }
}
