use std::sync::{Arc, Mutex, MutexGuard};

#[cfg(test)]
use crate::core::framework::render::RenderCapabilitySummary;
use crate::core::framework::render::{
    PlanarReflectionUpdateState, RenderEnvironmentCaptureHandle, RenderEnvironmentCapturePhase,
    RenderEnvironmentCaptureSourcePayload, RenderFrameProfile, RenderFrameworkError,
    RenderSubmissionConfig, ShaderVariantPrewarmManifest, UiRenderSubmission,
};
use crate::core::TaskPool;
#[cfg(test)]
use crate::core::{math::UVec2, resource::ResourceId};
#[cfg(test)]
use crate::graphics::shader::ShaderVariantCacheDisk;

use super::super::environment_capture_scheduler::{
    EnvironmentCapturePublication, EnvironmentCaptureScheduler, EnvironmentCaptureTransitionError,
    EnvironmentCaptureWorkItem,
};
use super::super::pipelined::{RenderSubmissionScheduler, RuntimeFrameSubmissionExecutor};
use super::super::render_framework_backend_error::render_framework_backend_error;
use super::super::render_framework_state::RenderFrameworkState;

pub(in crate::graphics::runtime::render_framework) struct WgpuRenderFrameworkCore {
    pub(in crate::graphics::runtime::render_framework) device_fault_gate:
        Arc<zr_rhi::DeviceFaultGate>,
    pub(in crate::graphics::runtime::render_framework) state: Mutex<RenderFrameworkState>,
    pub(in crate::graphics::runtime::render_framework) operation_lock: Mutex<()>,
    pub(in crate::graphics::runtime::render_framework) compute_task_pool: TaskPool,
    pub(in crate::graphics::runtime::render_framework) planar_reflection_updates:
        Mutex<PlanarReflectionUpdateState>,
    pub(in crate::graphics::runtime::render_framework) environment_captures:
        Mutex<EnvironmentCaptureScheduler>,
}

pub struct WgpuRenderFramework {
    // Drop the scheduler before the shared WGPU state so its worker can finish on
    // the owning thread without retaining the framework indefinitely.
    pub(in crate::graphics::runtime::render_framework) submission_scheduler:
        Mutex<RenderSubmissionScheduler>,
    pub(in crate::graphics::runtime::render_framework) core: Arc<WgpuRenderFrameworkCore>,
}

pub(in crate::graphics::runtime::render_framework) trait WgpuRenderFrameworkAccess {
    fn lock_operation(&self) -> MutexGuard<'_, ()>;
    fn lock_state(&self) -> MutexGuard<'_, RenderFrameworkState>;
    fn lock_planar_reflection_updates(&self) -> MutexGuard<'_, PlanarReflectionUpdateState>;
    fn compute_task_pool(&self) -> &TaskPool;
    fn begin_environment_capture_work_item(&self) -> Option<EnvironmentCaptureWorkItem>;
    fn advance_environment_capture_work_item(
        &self,
        handle: RenderEnvironmentCaptureHandle,
        phase: RenderEnvironmentCapturePhase,
        completed_work_items: u32,
    ) -> Result<(), EnvironmentCaptureTransitionError>;
    fn finish_environment_capture_work_item_failure(
        &self,
        handle: RenderEnvironmentCaptureHandle,
        diagnostic: String,
    ) -> Result<(), EnvironmentCaptureTransitionError>;
    fn settle_environment_capture_work_item_success(
        &self,
        handle: RenderEnvironmentCaptureHandle,
        source_payload: Option<RenderEnvironmentCaptureSourcePayload>,
        publication: &mut dyn FnMut(EnvironmentCapturePublication, &EnvironmentCaptureScheduler),
    ) -> Result<(), EnvironmentCaptureTransitionError>;
}

impl WgpuRenderFrameworkCore {
    pub(in crate::graphics::runtime::render_framework) fn ensure_device_admission(
        &self,
    ) -> Result<(), RenderFrameworkError> {
        self.device_fault_gate.ensure_admission().map_err(|error| {
            render_framework_backend_error(crate::graphics::GraphicsError::Rhi(
                zr_rhi::RhiError::DeviceAdmission(error),
            ))
        })
    }

    pub(in crate::graphics::runtime::render_framework) fn lock_operation(
        &self,
    ) -> MutexGuard<'_, ()> {
        #[cfg(feature = "profiling")]
        crate::profile_scope!("runtime", "render_framework.wait", "operation_lock");
        self.operation_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(in crate::graphics::runtime::render_framework) fn lock_state(
        &self,
    ) -> MutexGuard<'_, RenderFrameworkState> {
        #[cfg(feature = "profiling")]
        crate::profile_scope!("runtime", "render_framework.wait", "state");
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(in crate::graphics::runtime::render_framework) fn lock_planar_reflection_updates(
        &self,
    ) -> MutexGuard<'_, PlanarReflectionUpdateState> {
        match self.planar_reflection_updates.lock() {
            Ok(updates) => updates,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

impl WgpuRenderFrameworkAccess for WgpuRenderFrameworkCore {
    fn lock_operation(&self) -> MutexGuard<'_, ()> {
        WgpuRenderFrameworkCore::lock_operation(self)
    }

    fn lock_state(&self) -> MutexGuard<'_, RenderFrameworkState> {
        WgpuRenderFrameworkCore::lock_state(self)
    }

    fn lock_planar_reflection_updates(&self) -> MutexGuard<'_, PlanarReflectionUpdateState> {
        WgpuRenderFrameworkCore::lock_planar_reflection_updates(self)
    }

    fn compute_task_pool(&self) -> &TaskPool {
        &self.compute_task_pool
    }

    fn begin_environment_capture_work_item(&self) -> Option<EnvironmentCaptureWorkItem> {
        self.environment_captures
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .begin_next()
    }

    fn advance_environment_capture_work_item(
        &self,
        handle: RenderEnvironmentCaptureHandle,
        phase: RenderEnvironmentCapturePhase,
        completed_work_items: u32,
    ) -> Result<(), EnvironmentCaptureTransitionError> {
        self.environment_captures
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .advance_active(handle, phase, completed_work_items)
    }

    fn finish_environment_capture_work_item_failure(
        &self,
        handle: RenderEnvironmentCaptureHandle,
        diagnostic: String,
    ) -> Result<(), EnvironmentCaptureTransitionError> {
        self.environment_captures
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .finish_active_failure(handle, diagnostic)
    }

    fn settle_environment_capture_work_item_success(
        &self,
        handle: RenderEnvironmentCaptureHandle,
        source_payload: Option<RenderEnvironmentCaptureSourcePayload>,
        publication: &mut dyn FnMut(EnvironmentCapturePublication, &EnvironmentCaptureScheduler),
    ) -> Result<(), EnvironmentCaptureTransitionError> {
        self.environment_captures
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .finish_active_success_with_publication(handle, source_payload, publication)
    }
}

impl WgpuRenderFrameworkAccess for WgpuRenderFramework {
    fn lock_operation(&self) -> MutexGuard<'_, ()> {
        self.core.lock_operation()
    }

    fn lock_state(&self) -> MutexGuard<'_, RenderFrameworkState> {
        self.core.lock_state()
    }

    fn lock_planar_reflection_updates(&self) -> MutexGuard<'_, PlanarReflectionUpdateState> {
        self.core.lock_planar_reflection_updates()
    }

    fn compute_task_pool(&self) -> &TaskPool {
        &self.core.compute_task_pool
    }

    fn begin_environment_capture_work_item(&self) -> Option<EnvironmentCaptureWorkItem> {
        WgpuRenderFramework::begin_environment_capture_work_item(self)
    }

    fn advance_environment_capture_work_item(
        &self,
        handle: RenderEnvironmentCaptureHandle,
        phase: RenderEnvironmentCapturePhase,
        completed_work_items: u32,
    ) -> Result<(), EnvironmentCaptureTransitionError> {
        WgpuRenderFramework::advance_environment_capture_work_item(
            self,
            handle,
            phase,
            completed_work_items,
        )
    }

    fn finish_environment_capture_work_item_failure(
        &self,
        handle: RenderEnvironmentCaptureHandle,
        diagnostic: String,
    ) -> Result<(), EnvironmentCaptureTransitionError> {
        WgpuRenderFramework::finish_environment_capture_work_item_failure(self, handle, diagnostic)
    }

    fn settle_environment_capture_work_item_success(
        &self,
        handle: RenderEnvironmentCaptureHandle,
        source_payload: Option<RenderEnvironmentCaptureSourcePayload>,
        publication: &mut dyn FnMut(EnvironmentCapturePublication, &EnvironmentCaptureScheduler),
    ) -> Result<(), EnvironmentCaptureTransitionError> {
        WgpuRenderFramework::settle_environment_capture_work_item_success(
            self,
            handle,
            source_payload,
            publication,
        )
    }
}

impl WgpuRenderFramework {
    /// Moves one queued environment capture out of the control-plane mutex.
    ///
    /// The returned work item owns its scene snapshot. Callers may prepare
    /// resources and record GPU commands after this method returns without
    /// retaining the scheduler lock or taking the renderer state lock.
    pub(in crate::graphics) fn begin_environment_capture_work_item(
        &self,
    ) -> Option<EnvironmentCaptureWorkItem> {
        self.core
            .environment_captures
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .begin_next()
    }

    /// Publishes capture progress after the recorder has accepted its GPU work.
    ///
    /// This remains a scheduler-only operation. The recorder must not hold the
    /// renderer state or submission locks while reporting per-face progress.
    pub(in crate::graphics) fn advance_environment_capture_work_item(
        &self,
        handle: RenderEnvironmentCaptureHandle,
        phase: RenderEnvironmentCapturePhase,
        completed_work_items: u32,
    ) -> Result<(), EnvironmentCaptureTransitionError> {
        self.core
            .environment_captures
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .advance_active(handle, phase, completed_work_items)
    }

    /// Publishes physical output before exposing terminal success to the control plane.
    pub(in crate::graphics::runtime::render_framework) fn settle_environment_capture_work_item_success(
        &self,
        handle: RenderEnvironmentCaptureHandle,
        source_payload: Option<RenderEnvironmentCaptureSourcePayload>,
        publication: &mut dyn FnMut(EnvironmentCapturePublication, &EnvironmentCaptureScheduler),
    ) -> Result<(), EnvironmentCaptureTransitionError> {
        self.core
            .environment_captures
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .finish_active_success_with_publication(handle, source_payload, publication)
    }

    /// Records a recorder or GPU failure while preserving cancellation/supersession intent.
    pub(in crate::graphics) fn finish_environment_capture_work_item_failure(
        &self,
        handle: RenderEnvironmentCaptureHandle,
        diagnostic: impl Into<String>,
    ) -> Result<(), EnvironmentCaptureTransitionError> {
        self.core
            .environment_captures
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .finish_active_failure(handle, diagnostic)
    }

    /// Compiles an existing Plan08 manifest into the renderer-owned PSO caches.
    ///
    /// The caller owns inventory and dependency scanning. This hook only consumes
    /// the supplied immutable manifest during a loading or startup phase.
    pub fn prewarm_shader_pipelines(
        &self,
        manifest: &ShaderVariantPrewarmManifest,
    ) -> Result<
        crate::graphics::scene::RuntimeShaderPipelinePrewarmReport,
        crate::core::framework::render::RenderFrameworkError,
    > {
        self.finish_submission()?;
        let _operation_guard = self.lock_operation();
        Ok(self
            .lock_state()
            .renderer
            .prewarm_shader_pipelines(manifest))
    }

    /// Drains the renderer-owned timestamp result through the framework lock boundary.
    pub fn take_completed_gpu_timing_report(
        &self,
    ) -> Result<Option<crate::graphics::SceneRendererGpuTimingReport>, RenderFrameworkError> {
        self.finish_submission()?;
        let _operation_guard = self.lock_operation();
        let mut state = self.lock_state();
        let report = state.renderer.take_completed_gpu_timing_report();
        Ok(report.map(|report| {
            attach_matching_mesh_submission_profile(
                report,
                state.stats.last_resolved_gpu_frame_profile.as_deref(),
            )
        }))
    }

    /// Polls the specialized environment-only Base PSO without exposing the renderer owner.
    pub fn environment_only_pbr_base_pipeline_ready(&self) -> Result<bool, RenderFrameworkError> {
        self.finish_submission()?;
        let _operation_guard = self.lock_operation();
        let mut state = self.lock_state();
        state
            .renderer
            .environment_only_pbr_base_pipeline_ready()
            .map_err(render_framework_backend_error)
    }

    /// Retries bounded admission for the specialized environment-only Base PSO.
    pub fn retry_environment_only_pbr_base_pipeline_admission(
        &self,
    ) -> Result<(), RenderFrameworkError> {
        self.finish_submission()?;
        let _operation_guard = self.lock_operation();
        let mut state = self.lock_state();
        state
            .renderer
            .retry_environment_only_pbr_base_pipeline_admission()
            .map_err(render_framework_backend_error)
    }

    /// Polls the exact generic Forward Base PSO used by an explicitly queued
    /// non-default-IOR viewer material.
    pub fn pbr_ior_forward_base_pipeline_ready(&self) -> Result<bool, RenderFrameworkError> {
        self.finish_submission()?;
        let _operation_guard = self.lock_operation();
        let mut state = self.lock_state();
        state
            .renderer
            .pbr_ior_forward_base_pipeline_ready()
            .map_err(render_framework_backend_error)
    }

    /// Queues bounded nonblocking admission for the IOR generic Forward PSO.
    /// Repeated calls retry after compiler backpressure.
    pub fn queue_pbr_ior_forward_base_pipeline_admission(
        &self,
    ) -> Result<(), RenderFrameworkError> {
        self.finish_submission()?;
        let _operation_guard = self.lock_operation();
        let mut state = self.lock_state();
        state
            .renderer
            .queue_pbr_ior_forward_base_pipeline_admission()
            .map_err(render_framework_backend_error)
    }

    pub(in crate::graphics::runtime::render_framework) fn lock_operation(
        &self,
    ) -> MutexGuard<'_, ()> {
        self.core.lock_operation()
    }

    pub(in crate::graphics::runtime::render_framework) fn lock_state(
        &self,
    ) -> MutexGuard<'_, RenderFrameworkState> {
        self.core.lock_state()
    }

    pub(in crate::graphics::runtime::render_framework) fn lock_planar_reflection_updates(
        &self,
    ) -> MutexGuard<'_, PlanarReflectionUpdateState> {
        self.core.lock_planar_reflection_updates()
    }

    pub fn set_submission_config(
        &self,
        config: RenderSubmissionConfig,
    ) -> Result<(), crate::core::framework::render::RenderFrameworkError> {
        let mut scheduler = self
            .submission_scheduler
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // Drain before taking the operation lock: an active worker owns that lock until completion.
        scheduler.finish_pending()?;
        let _operation_guard = self.lock_operation();
        scheduler.set_config(Arc::clone(&self.core), config)?;
        let mut state = self.lock_state();
        state
            .renderer
            .set_gpu_pass_timing_enabled(config.allow_gpu_timing);
        state
            .renderer
            .set_async_pipeline_compile_enabled(config.async_pipeline_compile);
        state
            .renderer
            .set_parallel_recording(config.parallel_record, config.min_passes_per_bucket);
        state
            .renderer
            .set_hzb_diagnostics_readback_enabled(config.hzb_diagnostics_readback);
        Ok(())
    }

    pub fn submission_config(&self) -> RenderSubmissionConfig {
        self.submission_scheduler
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .config()
    }

    pub(in crate::graphics::runtime::render_framework) fn dispatch_submission(
        &self,
        execute: fn(
            &WgpuRenderFrameworkCore,
            crate::core::framework::render::RenderViewportHandle,
            crate::core::framework::render::RenderFrameExtract,
            Option<Arc<UiRenderSubmission>>,
        ) -> Result<(), crate::core::framework::render::RenderFrameworkError>,
        viewport: crate::core::framework::render::RenderViewportHandle,
        extract: crate::core::framework::render::RenderFrameExtract,
        ui: Option<Arc<UiRenderSubmission>>,
    ) -> Result<(), crate::core::framework::render::RenderFrameworkError> {
        self.submission_scheduler
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .submit(Arc::clone(&self.core), execute, viewport, extract, ui)
    }

    pub(in crate::graphics::runtime::render_framework) fn dispatch_runtime_frame_submission(
        &self,
        execute: RuntimeFrameSubmissionExecutor,
        viewport: crate::core::framework::render::RenderViewportHandle,
        frame: crate::graphics::types::ViewportRenderFrame,
    ) -> Result<(), crate::core::framework::render::RenderFrameworkError> {
        self.submission_scheduler
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .submit_runtime_frame(Arc::clone(&self.core), execute, viewport, frame)
    }

    pub(in crate::graphics::runtime::render_framework) fn finish_submission(
        &self,
    ) -> Result<(), crate::core::framework::render::RenderFrameworkError> {
        self.submission_scheduler
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .finish_pending()
    }

    /// Reports whether the active WGPU device exposes the timestamp path used
    /// by the compiled realtime IBL graph.
    pub fn realtime_ibl_gpu_timing_supported(&self) -> bool {
        let _operation_guard = self.lock_operation();
        let state = self.lock_state();
        state.renderer.realtime_ibl_gpu_timing_supported()
    }

    /// Returns the current realtime IBL readiness, generation identities, and
    /// optional retry or terminal failure without duplicating renderer policy.
    pub fn realtime_ibl_status_report(
        &self,
    ) -> Result<
        crate::graphics::RealtimeIblStatusReport,
        crate::core::framework::render::RenderFrameworkError,
    > {
        self.finish_submission()?;
        let _operation_guard = self.lock_operation();
        let state = self.lock_state();
        Ok(state.renderer.realtime_ibl_status_report())
    }

    /// Drains completed realtime IBL timestamp reports from compiled-frame
    /// submission. The framework owns the renderer lock and device lifetime.
    pub fn take_realtime_ibl_gpu_timing_reports(
        &self,
    ) -> Result<
        Vec<crate::graphics::RealtimeIblGpuTimingReport>,
        crate::core::framework::render::RenderFrameworkError,
    > {
        self.finish_submission()?;
        let _operation_guard = self.lock_operation();
        let mut state = self.lock_state();
        Ok(state.renderer.take_realtime_ibl_gpu_timing_reports())
    }

    /// Drains CPU recording reports accepted by realtime IBL submission. This
    /// is available during an active profiling capture on every WGPU adapter.
    pub fn take_realtime_ibl_cpu_timing_reports(
        &self,
    ) -> Result<
        Vec<crate::graphics::RealtimeIblCpuTimingReport>,
        crate::core::framework::render::RenderFrameworkError,
    > {
        self.finish_submission()?;
        let _operation_guard = self.lock_operation();
        let mut state = self.lock_state();
        Ok(state.renderer.take_realtime_ibl_cpu_timing_reports())
    }

    /// Invalidates one on-demand planar probe so the next camera loop submits
    /// its mirror capture before the owning main camera.
    pub fn request_planar_reflection_capture(&self, probe_id: u64) {
        self.lock_planar_reflection_updates().mark_dirty(probe_id);
    }

    #[cfg(test)]
    pub(crate) fn override_capabilities_for_tests(&self, capabilities: RenderCapabilitySummary) {
        self.lock_state().stats.capabilities = capabilities;
    }

    #[cfg(test)]
    pub(crate) fn request_next_created_viewport_graphics_debugger_capture_for_tests(&self) {
        self.lock_state()
            .graphics_debugger
            .request_next_created_viewport_capture();
    }

    #[cfg(test)]
    pub(crate) fn replace_shader_variant_disk_cache_for_tests(
        &self,
        cache: ShaderVariantCacheDisk,
    ) {
        self.lock_state()
            .renderer
            .replace_shader_variant_disk_cache_for_tests(cache);
    }

    #[cfg(test)]
    pub(crate) fn read_output_target_texture_rgba_for_tests(
        &self,
        texture_id: ResourceId,
    ) -> Result<Option<(UVec2, Vec<u8>)>, RenderFrameworkError> {
        let _operation_guard = self.lock_operation();
        let state = self.lock_state();
        state
            .renderer
            .read_output_target_texture_rgba_for_tests(&texture_id)
            .map_err(render_framework_backend_error)
    }

    #[cfg(test)]
    pub(crate) fn last_scene_velocity_readback_rg16_float_bytes_for_tests(
        &self,
    ) -> Option<Vec<u8>> {
        let _operation_guard = self.lock_operation();
        self.lock_state()
            .renderer
            .last_scene_velocity_readback_rg16_float_bytes()
    }

    #[cfg(test)]
    pub(crate) fn reflection_probe_upload_diagnostics_for_tests(
        &self,
    ) -> (usize, usize, usize, usize, Option<String>) {
        let _operation_guard = self.lock_operation();
        self.lock_state()
            .renderer
            .reflection_probe_upload_diagnostics_for_tests()
    }

    #[cfg(test)]
    pub(crate) fn reflection_probe_gpu_upload_diagnostics_for_tests(
        &self,
    ) -> Result<(u32, [[f32; 4]; 2], [[u16; 4]; 2]), RenderFrameworkError> {
        let _operation_guard = self.lock_operation();
        self.lock_state()
            .renderer
            .reflection_probe_gpu_upload_diagnostics_for_tests()
            .map_err(render_framework_backend_error)
    }
}

fn attach_matching_mesh_submission_profile(
    report: crate::graphics::SceneRendererGpuTimingReport,
    resolved_profile: Option<&RenderFrameProfile>,
) -> crate::graphics::SceneRendererGpuTimingReport {
    match resolved_profile.filter(|profile| profile.frame_generation == report.frame_generation()) {
        Some(profile) => report.with_mesh_submission_profile(profile.mesh_submission.clone()),
        None => report,
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::Arc;

    use crate::asset::pipeline::manager::ProjectAssetManager;
    use crate::core::framework::render::RenderSubmissionConfig;

    use super::*;

    #[test]
    fn realtime_ibl_timing_drain_finishes_pipelined_submission_before_state_access() {
        let source = include_str!("wgpu_render_framework.rs");
        let drain = source
            .split("pub fn take_realtime_ibl_gpu_timing_reports")
            .nth(1)
            .and_then(|source| {
                source
                    .split("/// Invalidates one on-demand planar probe")
                    .next()
            })
            .expect("realtime IBL timing drain");
        let finish_submission = drain
            .find("self.finish_submission()?;")
            .expect("timing drain must finish pending submission");
        let operation_lock = drain
            .find("let _operation_guard = self.lock_operation();")
            .expect("timing drain must serialize renderer access");

        assert!(finish_submission < operation_lock);
    }

    #[test]
    fn realtime_ibl_status_finishes_pipelined_submission_before_state_access() {
        let source = include_str!("wgpu_render_framework.rs");
        let status = source
            .split("pub fn realtime_ibl_status_report")
            .nth(1)
            .and_then(|source| {
                source
                    .split("/// Drains completed realtime IBL timestamp reports")
                    .next()
            })
            .expect("realtime IBL status report");
        let finish_submission = status
            .find("self.finish_submission()?;")
            .expect("status report must finish pending submission");
        let operation_lock = status
            .find("let _operation_guard = self.lock_operation();")
            .expect("status report must serialize renderer access");

        assert!(finish_submission < operation_lock);
        assert!(!source.contains("pub fn realtime_ibl_failure_report"));
    }

    #[test]
    fn realtime_ibl_cpu_timing_drain_finishes_pipelined_submission_before_state_access() {
        let source = include_str!("wgpu_render_framework.rs");
        let drain = source
            .split("pub fn take_realtime_ibl_cpu_timing_reports")
            .nth(1)
            .and_then(|source| {
                source
                    .split("/// Invalidates one on-demand planar probe")
                    .next()
            })
            .expect("realtime IBL CPU timing drain");
        let finish_submission = drain
            .find("self.finish_submission()?;")
            .expect("CPU timing drain must finish pending submission");
        let operation_lock = drain
            .find("let _operation_guard = self.lock_operation();")
            .expect("CPU timing drain must serialize renderer access");

        assert!(finish_submission < operation_lock);
    }

    #[test]
    fn device_admission_uses_the_shared_atomic_gate_without_renderer_state_access() {
        let source = include_str!("wgpu_render_framework.rs");
        let admission = source
            .split("fn ensure_device_admission")
            .nth(1)
            .and_then(|source| source.split("fn lock_operation").next())
            .expect("framework core must retain device admission");

        assert!(admission.contains("self.device_fault_gate"));
        assert!(admission.contains(".ensure_admission()"));
        assert!(
            !admission.contains("self.lock_state()"),
            "the healthy frame admission path must not take the renderer-state lock"
        );
    }

    #[test]
    fn frame_timing_drain_stays_inside_the_framework_lock_boundary() {
        let source = include_str!("wgpu_render_framework.rs");
        let drain = source
            .split("pub fn take_completed_gpu_timing_report")
            .nth(1)
            .and_then(|source| source.split("/// Polls the specialized").next())
            .expect("frame timing drain");
        let finish_submission = drain
            .find("self.finish_submission()?;")
            .expect("timing drain must finish pending submission");
        let operation_lock = drain
            .find("let _operation_guard = self.lock_operation();")
            .expect("timing drain must serialize renderer access");
        let renderer_drain = drain
            .find(".take_completed_gpu_timing_report()")
            .expect("timing drain must consume the renderer-owned report");

        assert!(finish_submission < operation_lock && operation_lock < renderer_drain);
    }

    #[test]
    fn frame_timing_report_attaches_only_a_same_generation_submission_snapshot() {
        let mesh_submission = crate::core::framework::render::RenderMeshSubmissionProfile {
            opaque_command_count: 2,
            advanced_pbr_opaque_command_count: 1,
            cached_command_hit_count: 4,
            command_rebuild_count: 0,
            dynamic_command_count: 1,
            ..Default::default()
        };
        let report = crate::graphics::SceneRendererGpuTimingReport::new(7, 1.0, []);
        let matching = RenderFrameProfile {
            frame_generation: 7,
            mesh_submission: mesh_submission.clone(),
            ..Default::default()
        };
        let mismatched = RenderFrameProfile {
            frame_generation: 8,
            mesh_submission,
            ..Default::default()
        };

        assert_eq!(
            attach_matching_mesh_submission_profile(report.clone(), Some(&matching))
                .mesh_submission_profile(),
            Some(&matching.mesh_submission)
        );
        assert_eq!(
            attach_matching_mesh_submission_profile(report, Some(&mismatched))
                .mesh_submission_profile(),
            None
        );
    }

    #[test]
    fn wgpu_render_framework_accessors_recover_poisoned_locks() {
        let framework = WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default()))
            .expect("framework should initialize for lock recovery test");

        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = framework.core.operation_lock.lock().unwrap();
            panic!("poison operation lock");
        }));
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = framework.core.state.lock().unwrap();
            panic!("poison render framework state lock");
        }));

        drop(framework.lock_operation());
        assert!(framework.lock_state().viewports.is_empty());
    }

    #[test]
    fn submission_config_switches_between_sync_and_pipelined_execution() {
        let framework = WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default()))
            .expect("framework should initialize for submission configuration test");

        assert_eq!(
            framework.submission_config(),
            RenderSubmissionConfig::synchronous()
        );
        assert!(!framework.lock_state().renderer.gpu_pass_timing_enabled());
        assert!(!framework
            .lock_state()
            .renderer
            .hzb_diagnostics_readback_enabled());
        assert_eq!(
            framework
                .lock_state()
                .renderer
                .parallel_record_min_passes_per_bucket(),
            None
        );
        let parallel_config = RenderSubmissionConfig::synchronous().with_parallel_recording(3);
        framework
            .set_submission_config(parallel_config)
            .expect("parallel recording configuration should reach the scene renderer");
        assert_eq!(framework.submission_config(), parallel_config);
        assert_eq!(
            framework
                .lock_state()
                .renderer
                .parallel_record_min_passes_per_bucket(),
            Some(3)
        );
        let timing_config = RenderSubmissionConfig::synchronous().with_gpu_timing();
        framework
            .set_submission_config(timing_config)
            .expect("GPU timing configuration should lazily create the timer when supported");
        assert_eq!(framework.submission_config(), timing_config);
        let state = framework.lock_state();
        assert_eq!(
            state.renderer.gpu_pass_timing_enabled(),
            state.stats.capabilities.supports_gpu_timestamp
        );
        drop(state);
        framework
            .set_submission_config(RenderSubmissionConfig::synchronous())
            .expect("disabling GPU timing should release the timer");
        assert!(!framework.lock_state().renderer.gpu_pass_timing_enabled());
        framework
            .set_submission_config(RenderSubmissionConfig::pipelined())
            .expect("pipelined configuration should initialize the worker");
        assert_eq!(
            framework.submission_config(),
            RenderSubmissionConfig::pipelined()
        );
        framework
            .set_submission_config(RenderSubmissionConfig::synchronous())
            .expect("synchronous configuration should drain and close the worker");

        let async_config = RenderSubmissionConfig::synchronous().with_async_pipeline_compile();
        framework
            .set_submission_config(async_config)
            .expect("async pipeline configuration should be accepted");
        assert!(framework
            .lock_state()
            .renderer
            .async_pipeline_compile_enabled());
        let hzb_readback_config =
            RenderSubmissionConfig::synchronous().with_hzb_diagnostics_readback();
        framework
            .set_submission_config(hzb_readback_config)
            .expect("HZB indirect readback configuration should be accepted");
        assert!(framework
            .lock_state()
            .renderer
            .hzb_diagnostics_readback_enabled());
    }
}
