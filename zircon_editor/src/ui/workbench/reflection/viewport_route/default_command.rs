use crate::ui::ViewportCommand;

pub(super) fn default_viewport_command(action_id: &str) -> Option<ViewportCommand> {
    match action_id {
        "pointer_move" => Some(ViewportCommand::PointerMoved { x: 0.0, y: 0.0 }),
        "left_press" => Some(ViewportCommand::LeftPressed { x: 0.0, y: 0.0 }),
        "left_release" => Some(ViewportCommand::LeftReleased),
        "right_press" => Some(ViewportCommand::RightPressed { x: 0.0, y: 0.0 }),
        "right_release" => Some(ViewportCommand::RightReleased),
        "middle_press" => Some(ViewportCommand::MiddlePressed { x: 0.0, y: 0.0 }),
        "middle_release" => Some(ViewportCommand::MiddleReleased),
        "scroll" => Some(ViewportCommand::Scrolled { delta: 0.0 }),
        "resize" => Some(ViewportCommand::Resized {
            width: 1,
            height: 1,
        }),
        "tool_drag" => Some(ViewportCommand::SetTool(
            zircon_scene::SceneViewportTool::Drag,
        )),
        "tool_move" => Some(ViewportCommand::SetTool(
            zircon_scene::SceneViewportTool::Move,
        )),
        "tool_rotate" => Some(ViewportCommand::SetTool(
            zircon_scene::SceneViewportTool::Rotate,
        )),
        "tool_scale" => Some(ViewportCommand::SetTool(
            zircon_scene::SceneViewportTool::Scale,
        )),
        "space_local" => Some(ViewportCommand::SetTransformSpace(
            zircon_scene::TransformSpace::Local,
        )),
        "space_global" => Some(ViewportCommand::SetTransformSpace(
            zircon_scene::TransformSpace::Global,
        )),
        "projection_perspective" => Some(ViewportCommand::SetProjectionMode(
            zircon_scene::ProjectionMode::Perspective,
        )),
        "projection_orthographic" => Some(ViewportCommand::SetProjectionMode(
            zircon_scene::ProjectionMode::Orthographic,
        )),
        "align_pos_x" => Some(ViewportCommand::AlignView(
            zircon_scene::ViewOrientation::PosX,
        )),
        "align_neg_x" => Some(ViewportCommand::AlignView(
            zircon_scene::ViewOrientation::NegX,
        )),
        "align_pos_y" => Some(ViewportCommand::AlignView(
            zircon_scene::ViewOrientation::PosY,
        )),
        "align_neg_y" => Some(ViewportCommand::AlignView(
            zircon_scene::ViewOrientation::NegY,
        )),
        "align_pos_z" => Some(ViewportCommand::AlignView(
            zircon_scene::ViewOrientation::PosZ,
        )),
        "align_neg_z" => Some(ViewportCommand::AlignView(
            zircon_scene::ViewOrientation::NegZ,
        )),
        "display_shaded" => Some(ViewportCommand::SetDisplayMode(
            zircon_scene::DisplayMode::Shaded,
        )),
        "display_wire_overlay" => Some(ViewportCommand::SetDisplayMode(
            zircon_scene::DisplayMode::WireOverlay,
        )),
        "display_wire_only" => Some(ViewportCommand::SetDisplayMode(
            zircon_scene::DisplayMode::WireOnly,
        )),
        "grid_hidden" => Some(ViewportCommand::SetGridMode(zircon_scene::GridMode::Hidden)),
        "grid_visible" => Some(ViewportCommand::SetGridMode(
            zircon_scene::GridMode::VisibleNoSnap,
        )),
        "grid_snap" => Some(ViewportCommand::SetGridMode(
            zircon_scene::GridMode::VisibleAndSnap,
        )),
        "gizmos_on" => Some(ViewportCommand::SetGizmosEnabled(true)),
        "gizmos_off" => Some(ViewportCommand::SetGizmosEnabled(false)),
        "lighting_on" => Some(ViewportCommand::SetPreviewLighting(true)),
        "lighting_off" => Some(ViewportCommand::SetPreviewLighting(false)),
        "skybox_on" => Some(ViewportCommand::SetPreviewSkybox(true)),
        "skybox_off" => Some(ViewportCommand::SetPreviewSkybox(false)),
        "frame_selection" => Some(ViewportCommand::FrameSelection),
        _ => None,
    }
}
