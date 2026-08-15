use crate::core::framework::render::{
    CorePipelineKind, ProjectionMode, RenderCameraClearColor, RenderCameraTarget,
    RenderViewportRect, DEFAULT_CAMERA_EXPOSURE_EV100, DEFAULT_CAMERA_MSAA_SAMPLES,
};
use crate::core::math::Real;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, zircon_reflect_derive::ZrReflect)]
#[zr_reflect(
    component,
    type_path = "zircon_runtime::scene::components::CameraComponent",
    script_visibility = "public"
)]
pub struct CameraComponent {
    /// Selects Core2d or Core3d without constraining perspective/orthographic projection.
    #[serde(default)]
    #[zr_reflect(skip)]
    pub core_pipeline: CorePipelineKind,
    #[serde(default)]
    #[zr_reflect(skip)]
    pub projection_mode: ProjectionMode,
    #[serde(default = "default_camera_fov_y_radians")]
    pub fov_y_radians: Real,
    #[serde(default = "default_camera_ortho_size")]
    #[zr_reflect(skip)]
    pub ortho_size: Real,
    #[serde(default = "default_camera_z_near")]
    pub z_near: Real,
    #[serde(default = "default_camera_z_far")]
    pub z_far: Real,
    #[serde(default)]
    #[zr_reflect(skip)]
    pub target: RenderCameraTarget,
    #[serde(default)]
    #[zr_reflect(skip)]
    pub viewport: Option<RenderViewportRect>,
    #[serde(default)]
    #[zr_reflect(skip)]
    pub order: i32,
    #[serde(default = "default_true")]
    #[zr_reflect(skip)]
    pub is_active: bool,
    #[serde(default)]
    #[zr_reflect(skip)]
    pub hdr: bool,
    #[serde(default = "default_camera_exposure_ev100")]
    #[zr_reflect(skip)]
    pub exposure_ev100: Real,
    #[serde(default)]
    #[zr_reflect(skip)]
    pub clear_color: RenderCameraClearColor,
    #[serde(default = "default_camera_msaa_samples")]
    #[zr_reflect(skip)]
    pub msaa_samples: u32,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            core_pipeline: CorePipelineKind::Core3d,
            projection_mode: ProjectionMode::Perspective,
            fov_y_radians: default_camera_fov_y_radians(),
            ortho_size: default_camera_ortho_size(),
            z_near: default_camera_z_near(),
            z_far: default_camera_z_far(),
            target: RenderCameraTarget::default(),
            viewport: None,
            order: 0,
            is_active: true,
            hdr: false,
            exposure_ev100: DEFAULT_CAMERA_EXPOSURE_EV100,
            clear_color: RenderCameraClearColor::default(),
            msaa_samples: DEFAULT_CAMERA_MSAA_SAMPLES,
        }
    }
}

const fn default_true() -> bool {
    true
}

const fn default_camera_ortho_size() -> Real {
    5.0
}

fn default_camera_fov_y_radians() -> Real {
    60.0_f32.to_radians()
}

const fn default_camera_z_near() -> Real {
    0.1
}

const fn default_camera_z_far() -> Real {
    200.0
}

const fn default_camera_exposure_ev100() -> Real {
    DEFAULT_CAMERA_EXPOSURE_EV100
}

const fn default_camera_msaa_samples() -> u32 {
    DEFAULT_CAMERA_MSAA_SAMPLES
}
