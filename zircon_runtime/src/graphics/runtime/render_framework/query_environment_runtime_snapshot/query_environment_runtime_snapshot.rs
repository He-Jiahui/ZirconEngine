use std::sync::Arc;

use crate::core::framework::render::{
    EnvironmentCaptureResidencyReport, EnvironmentRuntimeSnapshot, RenderFrameworkError,
};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn query_environment_runtime_snapshot(
    framework: &WgpuRenderFramework,
) -> Result<EnvironmentRuntimeSnapshot, RenderFrameworkError> {
    framework.finish_submission()?;
    let (
        frame_generation,
        frame_profile,
        scene_submission,
        reflection_probes,
        realtime_ibl,
        cubemap_upload,
        capture_residency,
        environment_ibl_hydration_cache,
    ) = {
        let _operation_guard = framework.lock_operation();
        let state = framework.lock_state();
        (
            state.stats.last_generation,
            Arc::clone(&state.stats.last_frame_profile),
            state.stats.last_scene_submission_completion_report,
            state.stats.last_reflection_probe_workload,
            state.renderer.realtime_ibl_status_report(),
            state.renderer.environment_cubemap_upload_report(),
            EnvironmentCaptureResidencyReport {
                observation_epoch: state.environment_capture_residency.observation_epoch(),
                last_published_handle: state.environment_capture_residency.last_published_handle(),
                last_published_output_generation: state
                    .environment_capture_residency
                    .last_published_output_generation(),
                resident_count: u32::try_from(state.environment_capture_residency.len())
                    .unwrap_or(u32::MAX),
                resident_gpu_bytes: state.environment_capture_residency.resident_gpu_bytes(),
                eviction_count: state.environment_capture_residency.eviction_count(),
            },
            Arc::clone(&state.environment_ibl_hydration_cache),
        )
    };
    let hydration = environment_ibl_hydration_cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .report();
    let capture = framework.environment_capture_report();

    Ok(EnvironmentRuntimeSnapshot::try_from_current_reports(
        frame_generation,
        &frame_profile,
        scene_submission,
        reflection_probes,
        realtime_ibl,
        hydration,
        capture,
        capture_residency,
        cubemap_upload,
    )?)
}

impl WgpuRenderFramework {
    pub fn query_environment_runtime_snapshot(
        &self,
    ) -> Result<EnvironmentRuntimeSnapshot, RenderFrameworkError> {
        query_environment_runtime_snapshot(self)
    }
}

#[cfg(test)]
#[path = "tests/query_environment_runtime_snapshot.rs"]
mod tests;
