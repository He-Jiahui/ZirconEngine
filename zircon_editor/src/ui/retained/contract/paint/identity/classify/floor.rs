use super::super::kind::ViewportSceneKind;

pub(super) fn floor_scene_kind(id: &str) -> Option<ViewportSceneKind> {
    if let Some(suffix) = id.strip_prefix("WorkbenchViewport") {
        match suffix {
            "FloorGrid" => return Some(ViewportSceneKind::FloorGrid),
            "FloorPanel" => return Some(ViewportSceneKind::FloorPanel),
            "FloorSeam" => return Some(ViewportSceneKind::FloorSeam),
            "FloorGrate" => return Some(ViewportSceneKind::FloorGrate),
            _ => {}
        }
    }

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

#[cfg(test)]
#[path = "tests/floor.rs"]
mod tests;
