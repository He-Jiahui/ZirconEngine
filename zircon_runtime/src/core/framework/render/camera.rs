use serde::{Deserialize, Serialize};
use crate::core::math::{Real, Transform, UVec2};

use crate::core::framework::scene::EntityId;

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
pub enum ProjectionMode {
    Perspective,
    Orthographic,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewportRenderSettings {
    pub projection_mode: ProjectionMode,
    pub display_mode: DisplayMode,
    pub preview_lighting: bool,
    pub preview_skybox: bool,
}

impl Default for ViewportRenderSettings {
    fn default() -> Self {
        Self {
            projection_mode: ProjectionMode::Perspective,
            display_mode: DisplayMode::Shaded,
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
    pub settings: ViewportRenderSettings,
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
