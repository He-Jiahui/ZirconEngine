use serde::{Deserialize, Serialize};
use zircon_framework::render::{
    DisplayMode, GridMode, ProjectionMode, SceneViewportTool, TransformSpace, ViewOrientation,
};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ViewportCommand {
    PointerMoved { x: f32, y: f32 },
    LeftPressed { x: f32, y: f32 },
    LeftReleased,
    RightPressed { x: f32, y: f32 },
    RightReleased,
    MiddlePressed { x: f32, y: f32 },
    MiddleReleased,
    Scrolled { delta: f32 },
    Resized { width: u32, height: u32 },
    SetTool(SceneViewportTool),
    SetTransformSpace(TransformSpace),
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
    FrameSelection,
}
