use crate::core::framework::render::{
    CorePipelineKind, ProjectionMode, RenderFrameExtract, ViewportCameraSnapshot,
};
use crate::core::math::Mat4;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FrameHistoryValidationKey {
    // Global history compatibility is structural. Per-pixel and per-domain content changes are
    // rejected by velocity, depth, reactive masks, and each history consumer's own metadata.
    world_identity: u64,
    camera: FrameHistoryCameraCompatibilityKey,
    effective_features: Vec<String>,
}

impl Default for FrameHistoryValidationKey {
    fn default() -> Self {
        Self {
            world_identity: 0,
            camera: FrameHistoryCameraCompatibilityKey::from(&ViewportCameraSnapshot::default()),
            effective_features: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct FrameHistoryCameraCompatibilityKey {
    core_pipeline: CorePipelineKind,
    projection_mode: ProjectionMode,
    projection_override: Option<Mat4>,
    hdr: bool,
    msaa_samples: u32,
}

impl From<&ViewportCameraSnapshot> for FrameHistoryCameraCompatibilityKey {
    fn from(camera: &ViewportCameraSnapshot) -> Self {
        Self {
            core_pipeline: camera.core_pipeline,
            projection_mode: camera.projection_mode,
            projection_override: camera.projection_override.clone(),
            hdr: camera.hdr,
            msaa_samples: camera.msaa_samples,
        }
    }
}

impl FrameHistoryValidationKey {
    pub(crate) fn from_extract(
        extract: &RenderFrameExtract,
        mut effective_features: Vec<String>,
    ) -> Self {
        effective_features.sort_unstable();
        effective_features.dedup();
        let camera = extract
            .view
            .selected_camera_descriptor()
            .map(|descriptor| &descriptor.camera)
            .unwrap_or(&extract.view.camera);

        Self {
            world_identity: extract.world.raw(),
            camera: FrameHistoryCameraCompatibilityKey::from(camera),
            effective_features,
        }
    }
}
