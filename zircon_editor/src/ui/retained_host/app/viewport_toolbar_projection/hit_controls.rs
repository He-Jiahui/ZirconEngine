use crate::ui::retained_host::SceneViewportChromeData;

pub(super) fn viewport_toolbar_hit_control_id(
    viewport: &SceneViewportChromeData,
    projection_control_id: &str,
) -> String {
    let control_id = match projection_control_id {
        "SetTool" => viewport_tool_action_id(viewport.tool.as_str()),
        "SetTransformSpace" => transform_space_action_id(viewport.transform_space.as_str()),
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

fn viewport_tool_action_id(tool: &str) -> Option<&'static str> {
    match tool {
        "Drag" => Some("tool.drag"),
        "Move" => Some("tool.move"),
        "Rotate" => Some("tool.rotate"),
        "Scale" => Some("tool.scale"),
        _ => None,
    }
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
