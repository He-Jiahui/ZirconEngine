use std::sync::Arc;

use thiserror::Error;

use super::realtime_ibl_status::RealtimeIblStatusReport;
use crate::core::framework::render::{
    IblBakeArtifactRequest, RenderEnvironmentCaptureHandle, RenderEnvironmentCaptureReport,
    RenderFrameProfile, RenderReflectionProbeWorkloadReport, RenderSceneSubmissionCompletionReport,
    SourceCubemapUploadKey,
};

pub const ENVIRONMENT_IBL_HYDRATION_REPORT_CAPACITY: usize = 4;

/// Bounded cache observation copied without exposing hydrated cubemap payloads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EnvironmentIblHydrationReport {
    pub observation_epoch: u64,
    pub resident_count: u32,
    pub pending_count: u32,
    /// Unique source, PMREM, and irradiance texel allocations retained by the cache.
    pub resident_decoded_texel_bytes: u64,
    /// Unique pre-encoded RGBA16F upload-row allocations retained by the cache.
    pub resident_prepared_upload_bytes: u64,
    /// Sum of decoded texels and prepared upload rows, excluding allocation metadata.
    pub resident_payload_bytes: u64,
    pub hit_count: u64,
    pub miss_count: u64,
    pub insert_count: u64,
    pub eviction_count: u64,
    pub reservation_count: u64,
    pub reservation_suppression_count: u64,
    pub reservation_release_count: u64,
    pub resident_requests:
        [Option<IblBakeArtifactRequest>; ENVIRONMENT_IBL_HYDRATION_REPORT_CAPACITY],
    pub pending_requests:
        [Option<IblBakeArtifactRequest>; ENVIRONMENT_IBL_HYDRATION_REPORT_CAPACITY],
}

/// Bounded observation of environment cubemap upload staging and publication state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EnvironmentCubemapUploadReport {
    pub observation_epoch: u64,
    pub committed_upload_key: SourceCubemapUploadKey,
    pub pending_upload_key: Option<SourceCubemapUploadKey>,
    /// Bytes copied into the frame upload batch by the latest observation.
    pub last_scheduled_upload_bytes: u64,
    pub last_scheduled_copy_count: u64,
    pub peak_scheduled_upload_bytes: u64,
    pub peak_scheduled_copy_count: u64,
    pub cumulative_scheduled_upload_bytes: u64,
    pub scheduled_upload_batch_count: u64,
    /// Current reusable host allocation retained by the staging arena.
    pub host_staging_capacity_bytes: u64,
    /// Current reusable GPU staging buffer allocation.
    pub gpu_staging_capacity_bytes: u64,
    /// Successful or failed staging observations in which host capacity increased.
    pub host_staging_growth_batch_count: u64,
    /// Staging observations in which the GPU buffer was replaced by a larger allocation.
    pub gpu_staging_growth_batch_count: u64,
    /// Logical RGBA16F texel bytes for the source cube's resident mip chain.
    pub resident_source_texture_bytes: u64,
    /// Logical RGBA16F texel bytes for the specular PMREM cube's resident mip chain.
    pub resident_specular_texture_bytes: u64,
    /// Logical RGBA16F texel bytes for the irradiance cube's resident mip.
    pub resident_irradiance_texture_bytes: u64,
    /// Sum of the three logical destination texture budgets above.
    pub resident_texture_bytes: u64,
}

/// Bounded logical GPU-residency observation for completed environment captures.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EnvironmentCaptureResidencyReport {
    /// Saturating monotonic epoch advanced when a complete filtered capture becomes resident.
    pub observation_epoch: u64,
    /// Scheduler handle for the newest complete output accepted by residency.
    pub last_published_handle: Option<RenderEnvironmentCaptureHandle>,
    /// Output generation paired with `last_published_handle`; this is not a frame generation.
    pub last_published_output_generation: Option<u64>,
    pub resident_count: u32,
    /// Logical bytes retained by completed filtered capture outputs.
    pub resident_gpu_bytes: u64,
    pub eviction_count: u64,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum EnvironmentRuntimeSnapshotError {
    #[error(
        "environment runtime snapshot frame generation {frame_generation} does not match profile generation {profile_generation}"
    )]
    FrameProfileGenerationMismatch {
        frame_generation: u64,
        profile_generation: u64,
    },
}

/// Current environment observations assembled from bounded subsystem reports.
///
/// Asynchronous reports retain their own source identities and epochs; this is not a globally atomic
/// snapshot. The current-frame profile is shared by `Arc` so querying does not deep-clone pass or
/// subsystem vectors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvironmentRuntimeSnapshot {
    pub frame_generation: Option<u64>,
    pub frame_profile: Option<Arc<RenderFrameProfile>>,
    pub scene_submission: RenderSceneSubmissionCompletionReport,
    pub reflection_probes: RenderReflectionProbeWorkloadReport,
    pub realtime_ibl: RealtimeIblStatusReport,
    pub hydration: EnvironmentIblHydrationReport,
    pub capture: RenderEnvironmentCaptureReport,
    pub capture_residency: EnvironmentCaptureResidencyReport,
    pub cubemap_upload: EnvironmentCubemapUploadReport,
}

impl EnvironmentRuntimeSnapshot {
    pub fn try_from_current_reports(
        frame_generation: Option<u64>,
        frame_profile: &Arc<RenderFrameProfile>,
        scene_submission: RenderSceneSubmissionCompletionReport,
        reflection_probes: RenderReflectionProbeWorkloadReport,
        realtime_ibl: RealtimeIblStatusReport,
        hydration: EnvironmentIblHydrationReport,
        capture: RenderEnvironmentCaptureReport,
        capture_residency: EnvironmentCaptureResidencyReport,
        cubemap_upload: EnvironmentCubemapUploadReport,
    ) -> Result<Self, EnvironmentRuntimeSnapshotError> {
        let frame_profile = match frame_generation {
            None => None,
            Some(frame_generation) if frame_profile.frame_generation == frame_generation => {
                Some(Arc::clone(frame_profile))
            }
            Some(frame_generation) => {
                return Err(
                    EnvironmentRuntimeSnapshotError::FrameProfileGenerationMismatch {
                        frame_generation,
                        profile_generation: frame_profile.frame_generation,
                    },
                );
            }
        };

        Ok(Self {
            frame_generation,
            frame_profile,
            scene_submission,
            reflection_probes,
            realtime_ibl,
            hydration,
            capture,
            capture_residency,
            cubemap_upload,
        })
    }
}

#[cfg(test)]
#[path = "tests/runtime_snapshot.rs"]
mod tests;
