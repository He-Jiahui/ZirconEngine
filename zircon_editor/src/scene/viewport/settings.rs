use serde::{Deserialize, Serialize};

use crate::scene::modes::SceneModeActivation;
use crate::scene::viewport::TransformHandleKind;
use zircon_runtime::core::framework::render::{
    DisplayMode, ProjectionMode, ViewportRenderSettings,
};
use zircon_runtime_interface::math::Real;

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
    pub transform_handle: TransformHandleKind,
    pub transform_space: TransformSpace,
    pub projection_mode: ProjectionMode,
    pub view_orientation: ViewOrientation,
    pub gizmos_enabled: bool,
    pub display_mode: DisplayMode,
    pub grid_mode: GridMode,
    pub preview_lighting: bool,
    pub preview_skybox: bool,
}

/// Resolved snap values supplied by the editor settings registry. This is a
/// value projection, not part of the serializable viewport or scene state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SceneViewportSnapSteps {
    pub(crate) translate_step: Real,
    pub(crate) rotate_step_deg: Real,
    pub(crate) scale_step: Real,
}

impl Default for SceneViewportSnapSteps {
    fn default() -> Self {
        Self {
            translate_step: 1.0,
            rotate_step_deg: 15.0,
            scale_step: 0.1,
        }
    }
}

/// Presentation projection that combines transient viewport controls with
/// snap values resolved from `SettingsRegistry` for a single UI frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneViewportChromeSettings {
    pub mode: SceneModeActivation,
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

impl SceneViewportChromeSettings {
    pub(crate) fn new(
        settings: &SceneViewportSettings,
        snap_steps: SceneViewportSnapSteps,
        mode: SceneModeActivation,
    ) -> Self {
        Self {
            mode,
            transform_space: settings.transform_space,
            projection_mode: settings.projection_mode,
            view_orientation: settings.view_orientation,
            gizmos_enabled: settings.gizmos_enabled,
            display_mode: settings.display_mode,
            grid_mode: settings.grid_mode,
            translate_step: snap_steps.translate_step,
            rotate_step_deg: snap_steps.rotate_step_deg,
            scale_step: snap_steps.scale_step,
            preview_lighting: settings.preview_lighting,
            preview_skybox: settings.preview_skybox,
        }
    }
}

impl Default for SceneViewportChromeSettings {
    fn default() -> Self {
        Self::new(
            &SceneViewportSettings::default(),
            SceneViewportSnapSteps::default(),
            SceneModeActivation::Transform(TransformHandleKind::Move),
        )
    }
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
            transform_handle: TransformHandleKind::Move,
            transform_space: TransformSpace::Local,
            projection_mode: ProjectionMode::Perspective,
            view_orientation: ViewOrientation::User,
            gizmos_enabled: true,
            display_mode: DisplayMode::Shaded,
            grid_mode: GridMode::VisibleNoSnap,
            preview_lighting: true,
            preview_skybox: true,
        }
    }
}
