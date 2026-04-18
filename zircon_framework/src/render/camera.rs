use serde::{Deserialize, Serialize};
use zircon_math::{Real, Transform, UVec2};

use crate::scene::EntityId;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewportCameraSnapshot {
    pub transform: Transform,
    pub projection_mode: ProjectionMode,
    pub fov_y_radians: Real,
    pub ortho_size: Real,
    pub z_near: Real,
    pub z_far: Real,
    pub aspect_ratio: Real,
}

impl ViewportCameraSnapshot {
    pub fn apply_viewport_size(&mut self, viewport_size: UVec2) {
        self.aspect_ratio = aspect_ratio_from_viewport_size(viewport_size);
    }
}

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
pub enum ProjectionMode {
    Perspective,
    Orthographic,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayMode {
    Shaded,
    WireOverlay,
    WireOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FallbackSkyboxKind {
    None,
    ProceduralGradient,
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

impl Default for ViewportCameraSnapshot {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            projection_mode: ProjectionMode::Perspective,
            fov_y_radians: 60.0_f32.to_radians(),
            ortho_size: 5.0,
            z_near: 0.1,
            z_far: 200.0,
            aspect_ratio: default_viewport_aspect_ratio(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneViewportExtractRequest {
    pub settings: SceneViewportSettings,
    pub selection: Vec<EntityId>,
    pub active_camera_override: Option<EntityId>,
    pub camera: Option<ViewportCameraSnapshot>,
    pub viewport_size: Option<UVec2>,
}

pub const fn default_viewport_aspect_ratio() -> Real {
    16.0 / 9.0
}

pub fn aspect_ratio_from_viewport_size(viewport_size: UVec2) -> Real {
    viewport_size.x.max(1) as Real / viewport_size.y.max(1) as Real
}
