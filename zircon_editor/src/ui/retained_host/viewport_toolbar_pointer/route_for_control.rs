use super::viewport_toolbar_pointer_route::ViewportToolbarPointerRoute;

pub(super) enum ViewportToolbarControlRoute<'a> {
    ActivateSceneMode(&'static str),
    ActivateCustomSceneMode(&'a str),
    SetTransformSpace(&'static str),
    CyclePivotMode,
    SetProjectionMode(&'static str),
    AlignView(&'static str),
    CycleDisplayMode,
    CycleGridMode,
    CycleTranslateSnap,
    CycleRotateSnapDegrees,
    CycleScaleSnap,
    TogglePreviewLighting,
    TogglePreviewSkybox,
    ToggleGizmosEnabled,
    FrameSelection,
    EnterPlayMode,
    ExitPlayMode,
}

pub(super) fn control_route_for_id(control_id: &str) -> Option<ViewportToolbarControlRoute<'_>> {
    let route = match control_id {
        "mode.select" => ViewportToolbarControlRoute::ActivateSceneMode("Select"),
        "mode.move" => ViewportToolbarControlRoute::ActivateSceneMode("Transform.Move"),
        "mode.rotate" => ViewportToolbarControlRoute::ActivateSceneMode("Transform.Rotate"),
        "mode.scale" => ViewportToolbarControlRoute::ActivateSceneMode("Transform.Scale"),
        "space.local" | "transform.local" => {
            ViewportToolbarControlRoute::SetTransformSpace("Local")
        }
        "space.global" | "transform.global" => {
            ViewportToolbarControlRoute::SetTransformSpace("Global")
        }
        "pivot.cycle" => ViewportToolbarControlRoute::CyclePivotMode,
        "projection.perspective" => ViewportToolbarControlRoute::SetProjectionMode("Perspective"),
        "projection.orthographic" => ViewportToolbarControlRoute::SetProjectionMode("Orthographic"),
        "align.pos_x" => ViewportToolbarControlRoute::AlignView("PosX"),
        "align.neg_x" => ViewportToolbarControlRoute::AlignView("NegX"),
        "align.pos_y" => ViewportToolbarControlRoute::AlignView("PosY"),
        "align.neg_y" => ViewportToolbarControlRoute::AlignView("NegY"),
        "align.pos_z" => ViewportToolbarControlRoute::AlignView("PosZ"),
        "align.neg_z" => ViewportToolbarControlRoute::AlignView("NegZ"),
        "display.cycle" => ViewportToolbarControlRoute::CycleDisplayMode,
        "grid.cycle" => ViewportToolbarControlRoute::CycleGridMode,
        "snap.translate" | "translate_snap.cycle" => {
            ViewportToolbarControlRoute::CycleTranslateSnap
        }
        "snap.rotate" | "rotate_snap.cycle" => ViewportToolbarControlRoute::CycleRotateSnapDegrees,
        "snap.scale" | "scale_snap.cycle" => ViewportToolbarControlRoute::CycleScaleSnap,
        "toggle.lighting" | "preview_lighting.toggle" => {
            ViewportToolbarControlRoute::TogglePreviewLighting
        }
        "toggle.skybox" | "preview_skybox.toggle" => {
            ViewportToolbarControlRoute::TogglePreviewSkybox
        }
        "toggle.gizmos" | "gizmos.toggle" => ViewportToolbarControlRoute::ToggleGizmosEnabled,
        "frame.selection" | "frame_selection" => ViewportToolbarControlRoute::FrameSelection,
        "EnterPlayMode" => ViewportToolbarControlRoute::EnterPlayMode,
        "ExitPlayMode" => ViewportToolbarControlRoute::ExitPlayMode,
        _ => {
            return control_id
                .strip_prefix("mode.custom:")
                .filter(|mode_id| !mode_id.is_empty())
                .map(ViewportToolbarControlRoute::ActivateCustomSceneMode);
        }
    };
    Some(route)
}

pub(super) fn validate_control_id(surface_key: &str, control_id: &str) -> Result<(), String> {
    control_route_for_id(control_id)
        .map(|_| ())
        .ok_or_else(|| unknown_control_error(surface_key, control_id))
}

pub(super) fn route_for_control(
    surface_key: &str,
    control_id: &str,
) -> Result<ViewportToolbarPointerRoute, String> {
    control_route_for_id(control_id)
        .map(|route| route.into_owned_route(surface_key))
        .ok_or_else(|| unknown_control_error(surface_key, control_id))
}

impl ViewportToolbarControlRoute<'_> {
    fn into_owned_route(self, surface_key: &str) -> ViewportToolbarPointerRoute {
        match self {
            Self::ActivateSceneMode(mode) => ViewportToolbarPointerRoute::ActivateSceneMode {
                surface_key: surface_key.to_string(),
                mode: mode.to_string(),
            },
            Self::ActivateCustomSceneMode(mode_id) => {
                ViewportToolbarPointerRoute::ActivateSceneMode {
                    surface_key: surface_key.to_string(),
                    mode: format!("Custom:{mode_id}"),
                }
            }
            Self::SetTransformSpace(space) => ViewportToolbarPointerRoute::SetTransformSpace {
                surface_key: surface_key.to_string(),
                space: space.to_string(),
            },
            Self::CyclePivotMode => ViewportToolbarPointerRoute::CyclePivotMode {
                surface_key: surface_key.to_string(),
            },
            Self::SetProjectionMode(mode) => ViewportToolbarPointerRoute::SetProjectionMode {
                surface_key: surface_key.to_string(),
                mode: mode.to_string(),
            },
            Self::AlignView(orientation) => ViewportToolbarPointerRoute::AlignView {
                surface_key: surface_key.to_string(),
                orientation: orientation.to_string(),
            },
            Self::CycleDisplayMode => ViewportToolbarPointerRoute::CycleDisplayMode {
                surface_key: surface_key.to_string(),
            },
            Self::CycleGridMode => ViewportToolbarPointerRoute::CycleGridMode {
                surface_key: surface_key.to_string(),
            },
            Self::CycleTranslateSnap => ViewportToolbarPointerRoute::CycleTranslateSnap {
                surface_key: surface_key.to_string(),
            },
            Self::CycleRotateSnapDegrees => ViewportToolbarPointerRoute::CycleRotateSnapDegrees {
                surface_key: surface_key.to_string(),
            },
            Self::CycleScaleSnap => ViewportToolbarPointerRoute::CycleScaleSnap {
                surface_key: surface_key.to_string(),
            },
            Self::TogglePreviewLighting => ViewportToolbarPointerRoute::TogglePreviewLighting {
                surface_key: surface_key.to_string(),
            },
            Self::TogglePreviewSkybox => ViewportToolbarPointerRoute::TogglePreviewSkybox {
                surface_key: surface_key.to_string(),
            },
            Self::ToggleGizmosEnabled => ViewportToolbarPointerRoute::ToggleGizmosEnabled {
                surface_key: surface_key.to_string(),
            },
            Self::FrameSelection => ViewportToolbarPointerRoute::FrameSelection {
                surface_key: surface_key.to_string(),
            },
            Self::EnterPlayMode => ViewportToolbarPointerRoute::EnterPlayMode {
                surface_key: surface_key.to_string(),
            },
            Self::ExitPlayMode => ViewportToolbarPointerRoute::ExitPlayMode {
                surface_key: surface_key.to_string(),
            },
        }
    }
}

fn unknown_control_error(surface_key: &str, control_id: &str) -> String {
    format!("Unknown viewport toolbar control {control_id} on surface {surface_key}")
}

#[cfg(test)]
#[path = "tests/route_for_control.rs"]
mod tests;
