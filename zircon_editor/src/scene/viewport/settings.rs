use serde::{Deserialize, Serialize};

use zircon_runtime::core::framework::render::{
    DisplayMode, ProjectionMode, ViewportRenderSettings,
};
use zircon_runtime::core::math::Real;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SceneViewportTool {
    Drag,
    Move,
    Rotate,
    Scale,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformSpace {
    Local,
    Global,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewOrientation {
    User,
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridMode {
    Hidden,
    VisibleNoSnap,
    VisibleAndSnap,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneViewportSettings {
    pub tool: SceneViewportTool,
    pub transform_space: TransformSpace,
    pub projection_mode: ProjectionMode,
    pub view_orientation: ViewOrientation,
    pub gizmos_enabled: bool,
    pub display_mode: DisplayMode,
    pub grid_mode: GridMode,
    pub translate_step: Real,
    pub rotate_step_deg: Real,
    pub scale_step: Real,
    pub preview_lighting: bool,
    pub preview_skybox: bool,
}

impl SceneViewportSettings {
    pub fn render_settings(&self) -> ViewportRenderSettings {
        ViewportRenderSettings {
            projection_mode: self.projection_mode,
            display_mode: self.display_mode,
            preview_lighting: self.preview_lighting,
            preview_skybox: self.preview_skybox,
        }
    }
}

impl Default for SceneViewportSettings {
    fn default() -> Self {
        Self {
            tool: SceneViewportTool::Move,
            transform_space: TransformSpace::Local,
            projection_mode: ProjectionMode::Perspective,
            view_orientation: ViewOrientation::User,
            gizmos_enabled: true,
            display_mode: DisplayMode::Shaded,
            grid_mode: GridMode::VisibleNoSnap,
            translate_step: 1.0,
            rotate_step_deg: 15.0,
            scale_step: 0.1,
            preview_lighting: true,
            preview_skybox: true,
        }
    }
}
