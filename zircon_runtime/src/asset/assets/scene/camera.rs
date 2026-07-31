use crate::asset::AssetReference;
use crate::core::framework::render::{
    CorePipelineKind, DEFAULT_CAMERA_EXPOSURE_EV100, DEFAULT_CAMERA_MSAA_SAMPLES, ProjectionMode,
    RenderCameraClearColor,
};
use crate::core::math::Real;
use serde::{Deserialize, Serialize};

use super::defaults::{
    default_camera_exposure_ev100, default_camera_fov_y_radians, default_camera_msaa_samples,
    default_camera_ortho_size, default_camera_z_far, default_camera_z_near, default_true,
    default_viewport_depth_max,
};
use super::post_process::ScenePostProcessSettingsAsset;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneCameraAsset {
    /// Authored render-pipeline identity; projection remains an independent matrix choice.
    #[serde(default)]
    pub core_pipeline: CorePipelineKind,
    #[serde(default)]
    pub projection_mode: ProjectionMode,
    #[serde(default = "default_camera_fov_y_radians")]
    pub fov_y_radians: Real,
    #[serde(default = "default_camera_ortho_size")]
    pub ortho_size: Real,
    #[serde(default = "default_camera_z_near")]
    pub z_near: Real,
    #[serde(default = "default_camera_z_far")]
    pub z_far: Real,
    #[serde(default)]
    pub target: SceneCameraTargetAsset,
    #[serde(default)]
    pub viewport: Option<SceneViewportRectAsset>,
    #[serde(default)]
    pub order: i32,
    #[serde(default = "default_true")]
    pub active: bool,
    #[serde(default)]
    pub hdr: bool,
    #[serde(default = "default_camera_exposure_ev100")]
    pub exposure_ev100: Real,
    #[serde(default)]
    pub clear_color: RenderCameraClearColor,
    #[serde(default = "default_camera_msaa_samples")]
    pub msaa_samples: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_process_settings: Option<ScenePostProcessSettingsAsset>,
}

impl Default for SceneCameraAsset {
    fn default() -> Self {
        Self {
            core_pipeline: CorePipelineKind::Core3d,
            projection_mode: ProjectionMode::Perspective,
            fov_y_radians: default_camera_fov_y_radians(),
            ortho_size: default_camera_ortho_size(),
            z_near: default_camera_z_near(),
            z_far: default_camera_z_far(),
            target: SceneCameraTargetAsset::default(),
            viewport: None,
            order: 0,
            active: true,
            hdr: false,
            exposure_ev100: DEFAULT_CAMERA_EXPOSURE_EV100,
            clear_color: RenderCameraClearColor::default(),
            msaa_samples: DEFAULT_CAMERA_MSAA_SAMPLES,
            post_process_settings: None,
        }
    }
}

impl SceneCameraAsset {
    pub fn direct_references(&self) -> Vec<AssetReference> {
        match &self.target {
            SceneCameraTargetAsset::Texture { texture } => vec![texture.clone()],
            SceneCameraTargetAsset::PrimarySurface | SceneCameraTargetAsset::Headless { .. } => {
                Vec::new()
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SceneCameraTargetAsset {
    PrimarySurface,
    Texture { texture: AssetReference },
    Headless { size: [u32; 2] },
}

impl Default for SceneCameraTargetAsset {
    fn default() -> Self {
        Self::PrimarySurface
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneViewportRectAsset {
    pub physical_position: [u32; 2],
    pub physical_size: [u32; 2],
    #[serde(default)]
    pub depth_min: Real,
    #[serde(default = "default_viewport_depth_max")]
    pub depth_max: Real,
}

impl Default for SceneViewportRectAsset {
    fn default() -> Self {
        Self {
            physical_position: [0, 0],
            physical_size: [1, 1],
            depth_min: 0.0,
            depth_max: 1.0,
        }
    }
}
