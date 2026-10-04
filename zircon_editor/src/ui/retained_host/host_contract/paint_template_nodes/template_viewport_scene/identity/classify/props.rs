use super::super::kind::ViewportSceneKind;

pub(super) fn prop_scene_kind(id: &str) -> Option<ViewportSceneKind> {
    if id == "WorkbenchViewportPropBody" {
        Some(ViewportSceneKind::PropBody)
    } else if id == "WorkbenchViewportPropTop" {
        Some(ViewportSceneKind::PropTop)
    } else if id.contains("Cargo") {
        if id.contains("Inner") {
            Some(ViewportSceneKind::CargoInner)
        } else {
            Some(ViewportSceneKind::Cargo)
        }
    } else if id.contains("Rack") {
        Some(ViewportSceneKind::Rack)
    } else {
        None
    }
}

#[cfg(test)]
#[path = "tests/props.rs"]
mod tests;
