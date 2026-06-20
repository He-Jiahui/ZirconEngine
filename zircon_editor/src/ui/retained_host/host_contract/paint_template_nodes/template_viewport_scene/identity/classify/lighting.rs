use super::super::kind::ViewportSceneKind;

pub(super) fn lighting_scene_kind(id: &str) -> Option<ViewportSceneKind> {
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
    } else {
        None
    }
}
