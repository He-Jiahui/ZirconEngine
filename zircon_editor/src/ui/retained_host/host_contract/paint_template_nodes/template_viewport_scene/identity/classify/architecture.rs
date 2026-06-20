use super::super::kind::ViewportSceneKind;

pub(super) fn architecture_scene_kind(id: &str) -> Option<ViewportSceneKind> {
    if id.contains("SideLeftStairs") {
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
    } else {
        None
    }
}
