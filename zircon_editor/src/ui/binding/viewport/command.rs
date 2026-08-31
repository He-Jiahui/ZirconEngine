use crate::scene::modes::SceneModeActivation;
use crate::scene::selection::SelectionMutation;
use crate::scene::viewport::{
    DisplayMode, GridMode, PivotMode, ProjectionMode, TransformSpace, ViewOrientation,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ViewportCommand {
    PointerMoved {
        x: f32,
        y: f32,
    },
    LeftPressed {
        x: f32,
        y: f32,
        selection_mutation: SelectionMutation,
    },
    LeftReleased,
    CancelInteraction,
    RightPressed {
        x: f32,
        y: f32,
    },
    RightReleased,
    MiddlePressed {
        x: f32,
        y: f32,
    },
    MiddleReleased,
    Scrolled {
        delta: f32,
    },
    Resized {
        width: u32,
        height: u32,
    },
    ActivateSceneMode(SceneModeActivation),
    SetTransformSpace(TransformSpace),
    SetPivotMode(PivotMode),
    SetProjectionMode(ProjectionMode),
    AlignView(ViewOrientation),
    SetDisplayMode(DisplayMode),
    SetGridMode(GridMode),
    SetTranslateSnap(f32),
    SetRotateSnapDegrees(f32),
    SetScaleSnap(f32),
    SetPreviewLighting(bool),
    SetPreviewSkybox(bool),
    SetGizmosEnabled(bool),
    ToggleOverlayProvider {
        provider_id: String,
    },
    FrameSelection,
}
