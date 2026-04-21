pub(super) fn projection_control_for_action(control_id: &str) -> Option<&'static str> {
    match control_id {
        "tool.drag" | "tool.move" | "tool.rotate" | "tool.scale" => Some("SetTool"),
        "space.local" | "space.global" | "transform.local" | "transform.global" => {
            Some("SetTransformSpace")
        }
        "projection.perspective" | "projection.orthographic" => Some("SetProjectionMode"),
        "align.pos_x" | "align.neg_x" | "align.pos_y" | "align.neg_y" | "align.pos_z"
        | "align.neg_z" => Some("AlignView"),
        "display.cycle" => Some("SetDisplayMode"),
        "grid.cycle" => Some("SetGridMode"),
        "snap.translate" | "translate_snap.cycle" => Some("SetTranslateSnap"),
        "snap.rotate" | "rotate_snap.cycle" => Some("SetRotateSnapDegrees"),
        "snap.scale" | "scale_snap.cycle" => Some("SetScaleSnap"),
        "toggle.lighting" | "preview_lighting.toggle" => Some("SetPreviewLighting"),
        "toggle.skybox" | "preview_skybox.toggle" => Some("SetPreviewSkybox"),
        "toggle.gizmos" | "gizmos.toggle" => Some("SetGizmosEnabled"),
        "frame.selection" | "frame_selection" => Some("FrameSelection"),
        _ => None,
    }
}
