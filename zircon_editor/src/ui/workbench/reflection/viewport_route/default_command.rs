use crate::scene::modes::SceneModeActivation;
use crate::scene::selection::SelectionMutation;
use crate::scene::viewport::{
    DisplayMode, GridMode, PivotMode, ProjectionMode, TransformHandleKind, TransformSpace,
    ViewOrientation,
};
use crate::ui::binding::ViewportCommand;

pub(super) fn default_viewport_command(action_id: &str) -> Option<ViewportCommand> {
    match action_id {
        "workbench.viewport.pointer.move" => Some(ViewportCommand::PointerMoved { x: 0.0, y: 0.0 }),
        "workbench.viewport.pointer.left.press" => Some(ViewportCommand::LeftPressed {
            x: 0.0,
            y: 0.0,
            selection_mutation: SelectionMutation::Replace,
        }),
        "workbench.viewport.pointer.left.release" => Some(ViewportCommand::LeftReleased),
        "workbench.viewport.interaction.cancel" => Some(ViewportCommand::CancelInteraction),
        "workbench.viewport.pointer.right.press" => {
            Some(ViewportCommand::RightPressed { x: 0.0, y: 0.0 })
        }
        "workbench.viewport.pointer.right.release" => Some(ViewportCommand::RightReleased),
        "workbench.viewport.pointer.middle.press" => {
            Some(ViewportCommand::MiddlePressed { x: 0.0, y: 0.0 })
        }
        "workbench.viewport.pointer.middle.release" => Some(ViewportCommand::MiddleReleased),
        "workbench.viewport.scroll" => Some(ViewportCommand::Scrolled { delta: 0.0 }),
        "workbench.viewport.resize" => Some(ViewportCommand::Resized {
            width: 1,
            height: 1,
        }),
        "workbench.viewport.mode.select" => Some(ViewportCommand::ActivateSceneMode(
            SceneModeActivation::Select,
        )),
        "workbench.viewport.mode.move" => Some(ViewportCommand::ActivateSceneMode(
            SceneModeActivation::Transform(TransformHandleKind::Move),
        )),
        "workbench.viewport.mode.rotate" => Some(ViewportCommand::ActivateSceneMode(
            SceneModeActivation::Transform(TransformHandleKind::Rotate),
        )),
        "workbench.viewport.mode.scale" => Some(ViewportCommand::ActivateSceneMode(
            SceneModeActivation::Transform(TransformHandleKind::Scale),
        )),
        "workbench.viewport.transform_space.local" => {
            Some(ViewportCommand::SetTransformSpace(TransformSpace::Local))
        }
        "workbench.viewport.transform_space.global" => {
            Some(ViewportCommand::SetTransformSpace(TransformSpace::Global))
        }
        "workbench.viewport.pivot.primary" => {
            Some(ViewportCommand::SetPivotMode(PivotMode::Primary))
        }
        "workbench.viewport.pivot.centroid" => {
            Some(ViewportCommand::SetPivotMode(PivotMode::Centroid))
        }
        "workbench.viewport.projection.perspective" => Some(ViewportCommand::SetProjectionMode(
            ProjectionMode::Perspective,
        )),
        "workbench.viewport.projection.orthographic" => Some(ViewportCommand::SetProjectionMode(
            ProjectionMode::Orthographic,
        )),
        "workbench.viewport.align.pos_x" => Some(ViewportCommand::AlignView(ViewOrientation::PosX)),
        "workbench.viewport.align.neg_x" => Some(ViewportCommand::AlignView(ViewOrientation::NegX)),
        "workbench.viewport.align.pos_y" => Some(ViewportCommand::AlignView(ViewOrientation::PosY)),
        "workbench.viewport.align.neg_y" => Some(ViewportCommand::AlignView(ViewOrientation::NegY)),
        "workbench.viewport.align.pos_z" => Some(ViewportCommand::AlignView(ViewOrientation::PosZ)),
        "workbench.viewport.align.neg_z" => Some(ViewportCommand::AlignView(ViewOrientation::NegZ)),
        "workbench.viewport.display.shaded" => {
            Some(ViewportCommand::SetDisplayMode(DisplayMode::Shaded))
        }
        "workbench.viewport.display.wire_overlay" => {
            Some(ViewportCommand::SetDisplayMode(DisplayMode::WireOverlay))
        }
        "workbench.viewport.display.wire_only" => {
            Some(ViewportCommand::SetDisplayMode(DisplayMode::WireOnly))
        }
        "workbench.viewport.grid.hidden" => Some(ViewportCommand::SetGridMode(GridMode::Hidden)),
        "workbench.viewport.grid.visible" => {
            Some(ViewportCommand::SetGridMode(GridMode::VisibleNoSnap))
        }
        "workbench.viewport.grid.snap" => {
            Some(ViewportCommand::SetGridMode(GridMode::VisibleAndSnap))
        }
        "workbench.viewport.gizmos.on" => Some(ViewportCommand::SetGizmosEnabled(true)),
        "workbench.viewport.gizmos.off" => Some(ViewportCommand::SetGizmosEnabled(false)),
        "workbench.viewport.lighting.on" => Some(ViewportCommand::SetPreviewLighting(true)),
        "workbench.viewport.lighting.off" => Some(ViewportCommand::SetPreviewLighting(false)),
        "workbench.viewport.skybox.on" => Some(ViewportCommand::SetPreviewSkybox(true)),
        "workbench.viewport.skybox.off" => Some(ViewportCommand::SetPreviewSkybox(false)),
        "workbench.viewport.selection.frame" => Some(ViewportCommand::FrameSelection),
        _ => None,
    }
}
