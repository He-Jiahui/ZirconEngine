use crate::ui::retained_host::SceneViewportChromeData;

/// Keep this key limited to fields read by `viewport_toolbar_hit_control_id`.
pub(super) fn viewport_toolbar_hit_route_key(viewport: &SceneViewportChromeData) -> [&str; 4] {
    [
        viewport.mode.as_str(),
        viewport.transform_space.as_str(),
        viewport.projection_mode.as_str(),
        viewport.view_orientation.as_str(),
    ]
}

pub(super) fn viewport_toolbar_hit_control_id(
    viewport: &SceneViewportChromeData,
    projection_control_id: &str,
) -> String {
    if projection_control_id == "ActivateSceneMode" {
        return scene_mode_action_id(viewport.mode.as_str())
            .unwrap_or_else(|| projection_control_id.to_string());
    }
    let control_id = match projection_control_id {
        "SetTransformSpace" => transform_space_action_id(viewport.transform_space.as_str()),
        "SetPivotMode" => Some("pivot.cycle"),
        "SetProjectionMode" => Some(projection_mode_action_id(viewport.projection_mode.as_str())),
        "AlignView" => Some(align_view_action_id(viewport.view_orientation.as_str())),
        "SetDisplayMode" => Some("display.cycle"),
        "SetGridMode" => Some("grid.cycle"),
        "SetTranslateSnap" => Some("snap.translate"),
        "SetRotateSnapDegrees" => Some("snap.rotate"),
        "SetScaleSnap" => Some("snap.scale"),
        "SetPreviewLighting" => Some("toggle.lighting"),
        "SetPreviewSkybox" => Some("toggle.skybox"),
        "SetGizmosEnabled" => Some("toggle.gizmos"),
        "FrameSelection" => Some("frame.selection"),
        "EnterPlayMode" => Some("EnterPlayMode"),
        "ExitPlayMode" => Some("ExitPlayMode"),
        _ => None,
    };
    control_id
        .map(str::to_string)
        .unwrap_or_else(|| projection_control_id.to_string())
}

fn scene_mode_action_id(mode: &str) -> Option<String> {
    let builtin = match mode {
        "Select" => Some("mode.select"),
        "Transform.Move" => Some("mode.move"),
        "Transform.Rotate" => Some("mode.rotate"),
        "Transform.Scale" => Some("mode.scale"),
        _ => None,
    };
    builtin.map(str::to_string).or_else(|| {
        mode.strip_prefix("Custom:")
            .filter(|mode_id| !mode_id.is_empty())
            .map(|mode_id| format!("mode.custom:{mode_id}"))
    })
}

fn transform_space_action_id(space: &str) -> Option<&'static str> {
    match space {
        "Local" => Some("space.local"),
        "Global" => Some("space.global"),
        _ => None,
    }
}

fn projection_mode_action_id(mode: &str) -> &'static str {
    match mode {
        "Orthographic" => "projection.orthographic",
        _ => "projection.perspective",
    }
}

fn align_view_action_id(orientation: &str) -> &'static str {
    match orientation {
        "PosX" => "align.pos_x",
        "NegX" => "align.neg_x",
        "PosY" => "align.pos_y",
        "NegY" => "align.neg_y",
        "PosZ" => "align.pos_z",
        _ => "align.neg_z",
    }
}
