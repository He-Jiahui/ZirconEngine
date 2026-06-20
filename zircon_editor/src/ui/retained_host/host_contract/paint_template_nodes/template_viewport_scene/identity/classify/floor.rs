use super::super::kind::ViewportSceneKind;

pub(super) fn floor_scene_kind(id: &str) -> Option<ViewportSceneKind> {
    if id.contains("Grid") {
        Some(ViewportSceneKind::FloorGrid)
    } else if id.contains("FloorPanel") {
        Some(ViewportSceneKind::FloorPanel)
    } else if id.contains("FloorSeam") {
        Some(ViewportSceneKind::FloorSeam)
    } else if id.contains("FloorGrate") {
        Some(ViewportSceneKind::FloorGrate)
    } else {
        None
    }
}
