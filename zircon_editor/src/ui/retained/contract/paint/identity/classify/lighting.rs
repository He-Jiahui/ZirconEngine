use super::super::kind::ViewportSceneKind;

pub(super) fn lighting_scene_kind(id: &str) -> Option<ViewportSceneKind> {
    if let Some(suffix) = id.strip_prefix("WorkbenchViewport") {
        match suffix {
            "Lightwash" => return Some(ViewportSceneKind::SoftLight),
            "Shadow" => return Some(ViewportSceneKind::SoftShadow),
            "FloorReflection" => return Some(ViewportSceneKind::FloorReflection),
            "WallLight" => return Some(ViewportSceneKind::WallLight),
            "Beacon" => return Some(ViewportSceneKind::Beacon),
            _ => {}
        }
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
    } else {
        None
    }
}

#[cfg(test)]
#[path = "tests/lighting.rs"]
mod tests;
