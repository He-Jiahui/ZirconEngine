use crate::core::math::{Mat4, Real, Transform, UVec2};
use serde::{Deserialize, Serialize};

use super::super::view_family::MAX_RENDER_RESOLUTION_FRACTION;
use super::super::{
    CorePipelineKind, RenderResolutionPolicy, RenderUpscalerKind, RenderViewFamilyPipeline,
    TemporalJitterSample,
};
use super::defaults::{
    aspect_ratio_from_viewport_size, default_camera_exposure_ev100, default_camera_msaa_samples,
    default_true,
};
use super::{
    default_viewport_aspect_ratio, ProjectionMode, RenderDynamicResolutionSettings,
    DEFAULT_CAMERA_EXPOSURE_EV100, DEFAULT_CAMERA_MSAA_SAMPLES,
};

// Implausible frame-to-frame camera deltas are treated as cuts. These limits are shared by
// temporal-history publication and camera-velocity generation so both domains reject together.
const TEMPORAL_CAMERA_MAX_TRANSLATION_FAR_PLANE_FRACTION: Real = 0.2;
const TEMPORAL_CAMERA_MAX_ROTATION_RADIANS: Real = core::f32::consts::FRAC_PI_3;
const TEMPORAL_CAMERA_MAX_FOV_DELTA_RADIANS: Real = core::f32::consts::PI / 12.0;
const TEMPORAL_CAMERA_MIN_FOV_RADIANS: Real = core::f32::consts::PI / 180.0;
const TEMPORAL_CAMERA_MAX_FOV_RADIANS: Real =
    core::f32::consts::PI - TEMPORAL_CAMERA_MIN_FOV_RADIANS;
const TEMPORAL_CAMERA_MAX_ORTHO_SIZE_RELATIVE_DELTA: Real = 0.25;
const TEMPORAL_CAMERA_MAX_CLIP_PLANE_RELATIVE_DELTA: Real = 0.5;
const TEMPORAL_CAMERA_MIN_PROJECTION_PARAMETER: Real = 0.001;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewportCameraSnapshot {
    pub transform: Transform,
    /// Selects the render schedule independently from the projection matrix.
    #[serde(default)]
    pub core_pipeline: CorePipelineKind,
    pub projection_mode: ProjectionMode,
    pub fov_y_radians: Real,
    pub ortho_size: Real,
    pub z_near: Real,
    pub z_far: Real,
    pub aspect_ratio: Real,
    #[serde(default)]
    pub projection_override: Option<Mat4>,
    #[serde(default = "default_true")]
    pub is_active: bool,
    #[serde(default)]
    pub hdr: bool,
    #[serde(default = "default_camera_exposure_ev100")]
    pub exposure_ev100: Real,
    #[serde(default = "default_camera_msaa_samples")]
    pub msaa_samples: u32,
    #[serde(default)]
    pub dynamic_resolution: RenderDynamicResolutionSettings,
    #[serde(default)]
    pub temporal_jitter: TemporalJitterSample,
}

impl ViewportCameraSnapshot {
    pub fn apply_viewport_size(&mut self, viewport_size: UVec2) {
        self.aspect_ratio =
            aspect_ratio_from_viewport_size(self.effective_viewport_size(viewport_size));
    }

    pub fn core_pipeline_kind(&self) -> CorePipelineKind {
        self.core_pipeline
    }

    pub fn effective_viewport_size(&self, target_size: UVec2) -> UVec2 {
        target_size
    }

    pub fn effective_render_size(&self, target_size: UVec2) -> UVec2 {
        self.render_view_family_pipeline(target_size, RenderUpscalerKind::Spatial)
            .resolution()
            .primary_extent()
    }

    pub fn supports_temporal_reprojection_from(&self, previous: &Self) -> bool {
        if self.projection_mode != previous.projection_mode
            || self.dynamic_resolution != previous.dynamic_resolution
            || !temporal_camera_clip_range_valid(self)
            || !temporal_camera_clip_range_valid(previous)
            || !temporal_camera_relative_parameter_compatible(
                self.z_near,
                previous.z_near,
                TEMPORAL_CAMERA_MAX_CLIP_PLANE_RELATIVE_DELTA,
            )
            || !temporal_camera_relative_parameter_compatible(
                self.z_far,
                previous.z_far,
                TEMPORAL_CAMERA_MAX_CLIP_PLANE_RELATIVE_DELTA,
            )
            || !temporal_camera_projection_shape_compatible(self, previous)
        {
            return false;
        }

        let translation_delta = self
            .transform
            .translation
            .distance(previous.transform.translation);
        if !translation_delta.is_finite() {
            return false;
        }
        let far_plane = self
            .z_far
            .min(previous.z_far)
            .max(self.z_near.max(previous.z_near));
        let max_translation_delta =
            (far_plane * TEMPORAL_CAMERA_MAX_TRANSLATION_FAR_PLANE_FRACTION).max(0.001);
        if translation_delta > max_translation_delta {
            return false;
        }

        let rotation_delta = self
            .transform
            .rotation
            .angle_between(previous.transform.rotation);
        rotation_delta.is_finite() && rotation_delta <= TEMPORAL_CAMERA_MAX_ROTATION_RADIANS
    }

    /// Converts the legacy camera scale into the primary fraction of a view-family plan.
    ///
    /// The upscaler category remains a render-graph decision: a camera scale alone must not
    /// silently turn TAA, a vendor SDK, or temporal history on.
    pub fn render_view_family_pipeline(
        &self,
        target_size: UVec2,
        upscaler: RenderUpscalerKind,
    ) -> RenderViewFamilyPipeline {
        RenderViewFamilyPipeline::resolve(
            self.effective_viewport_size(target_size),
            RenderResolutionPolicy::with_scales(
                self.dynamic_resolution.clamped_scale(),
                MAX_RENDER_RESOLUTION_FRACTION,
            ),
            upscaler,
        )
    }
}

fn temporal_camera_projection_shape_compatible(
    current: &ViewportCameraSnapshot,
    previous: &ViewportCameraSnapshot,
) -> bool {
    match (
        current.projection_override.as_ref(),
        previous.projection_override.as_ref(),
    ) {
        (Some(current), Some(previous)) => current == previous,
        (None, None) => match current.projection_mode {
            ProjectionMode::Perspective => {
                let fov_delta = (current.fov_y_radians - previous.fov_y_radians).abs();
                temporal_camera_fov_valid(current.fov_y_radians)
                    && temporal_camera_fov_valid(previous.fov_y_radians)
                    && fov_delta.is_finite()
                    && fov_delta <= TEMPORAL_CAMERA_MAX_FOV_DELTA_RADIANS
            }
            ProjectionMode::Orthographic => temporal_camera_relative_parameter_compatible(
                current.ortho_size,
                previous.ortho_size,
                TEMPORAL_CAMERA_MAX_ORTHO_SIZE_RELATIVE_DELTA,
            ),
        },
        _ => false,
    }
}

fn temporal_camera_clip_range_valid(camera: &ViewportCameraSnapshot) -> bool {
    camera.z_near.is_finite()
        && camera.z_far.is_finite()
        && camera.z_near > 0.0
        && camera.z_far > camera.z_near
}

fn temporal_camera_fov_valid(fov_y_radians: Real) -> bool {
    fov_y_radians.is_finite()
        && fov_y_radians >= TEMPORAL_CAMERA_MIN_FOV_RADIANS
        && fov_y_radians <= TEMPORAL_CAMERA_MAX_FOV_RADIANS
}

fn temporal_camera_relative_parameter_compatible(
    current: Real,
    previous: Real,
    max_delta_fraction: Real,
) -> bool {
    if !current.is_finite() || !previous.is_finite() || current <= 0.0 || previous <= 0.0 {
        return false;
    }

    let baseline = current
        .abs()
        .max(previous.abs())
        .max(TEMPORAL_CAMERA_MIN_PROJECTION_PARAMETER);
    ((current - previous).abs() / baseline) <= max_delta_fraction
}

impl Default for ViewportCameraSnapshot {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            core_pipeline: CorePipelineKind::Core3d,
            projection_mode: ProjectionMode::Perspective,
            fov_y_radians: 60.0_f32.to_radians(),
            ortho_size: 5.0,
            z_near: 0.1,
            z_far: 200.0,
            aspect_ratio: default_viewport_aspect_ratio(),
            projection_override: None,
            is_active: true,
            hdr: false,
            exposure_ev100: DEFAULT_CAMERA_EXPOSURE_EV100,
            msaa_samples: DEFAULT_CAMERA_MSAA_SAMPLES,
            dynamic_resolution: RenderDynamicResolutionSettings::default(),
            temporal_jitter: TemporalJitterSample::default(),
        }
    }
}
