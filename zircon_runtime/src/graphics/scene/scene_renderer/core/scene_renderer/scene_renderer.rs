use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use crate::core::framework::render::{
    FrameHistoryHandle, RenderFrameSubmissionReceipt, RenderGpuTimingStatus, RenderMeshBounds,
    RenderMeshSnapshot, RenderMeshSubmissionProfile,
};

use super::super::scene_renderer_core::SceneRendererCore;
use super::super::scene_submission_completion_journal::SceneSubmissionCompletionJournal;
use crate::graphics::backend::{
    GpuPassTimer, GpuPipelineStatisticsFrameResult, GpuPipelineStatisticsTimer,
    GpuTimerFrameResult, OffscreenTarget, RenderBackend,
};
use crate::graphics::scene::gpu_scene::{
    GpuSceneJournalConsumer, GpuSceneJournalOwnedStagedTransaction,
};
use crate::graphics::scene::render_scene::RenderSceneRegistry;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::mesh::PreparedMeshQueueStats;
use crate::graphics::scene::scene_renderer::sprite::PreparedSpriteQueueStats;
use crate::graphics::types::GraphicsError;
use crate::graphics::{
    RuntimePrepareMeshGeometrySeed, RuntimePrepareMeshSdfDeformationReason,
    RuntimePrepareMeshSdfSeed,
};

use super::super::super::graph_execution::{
    RenderGraphExecutionRecord, RenderPassExecutorRegistry,
};
use super::advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;

pub struct SceneRenderer {
    pub(in crate::graphics::scene::scene_renderer::core) core: SceneRendererCore,
    pub(in crate::graphics::scene::scene_renderer::core) streamer: ResourceStreamer,
    pub(in crate::graphics::scene::scene_renderer::core) render_scene_registry: RenderSceneRegistry,
    pub(in crate::graphics::scene::scene_renderer::core) gpu_scene_journal_consumers:
        HashMap<u64, GpuSceneJournalConsumer>,
    pub(in crate::graphics::scene::scene_renderer::core) pending_gpu_scene_journals:
        VecDeque<GpuSceneJournalOwnedStagedTransaction>,
    pub(in crate::graphics::scene::scene_renderer::core) target: Option<OffscreenTarget>,
    pub(in crate::graphics::scene::scene_renderer::core) last_capture_target:
        Option<SceneRendererCaptureTarget>,
    pub(in crate::graphics::scene::scene_renderer::core) history_targets:
        HashMap<FrameHistoryHandle, SceneFrameHistoryTextures>,
    pub(in crate::graphics::scene::scene_renderer::core) generation: u64,
    pub(in crate::graphics::scene::scene_renderer::core) last_frame_submission_receipt:
        Option<RenderFrameSubmissionReceipt>,
    pub(in crate::graphics::scene::scene_renderer::core) scene_submission_completion_journal:
        SceneSubmissionCompletionJournal,
    pub(in crate::graphics::scene::scene_renderer::core) gpu_pass_timing_requested: bool,
    pub(in crate::graphics::scene::scene_renderer::core) gpu_pass_timer: Option<GpuPassTimer>,
    pub(in crate::graphics::scene::scene_renderer::core) gpu_pipeline_statistics_timer:
        Option<GpuPipelineStatisticsTimer>,
    pub(in crate::graphics::scene::scene_renderer::core) last_gpu_timer_frame_result:
        Option<GpuTimerFrameResult>,
    pub(in crate::graphics::scene::scene_renderer::core) last_gpu_timing_status:
        RenderGpuTimingStatus,
    pub(in crate::graphics::scene::scene_renderer::core) last_gpu_pipeline_statistics_frame_result:
        Option<GpuPipelineStatisticsFrameResult>,
    pub(in crate::graphics::scene::scene_renderer::core) render_pass_executors:
        RenderPassExecutorRegistry,
    pub(in crate::graphics::scene::scene_renderer::core) last_render_graph_execution:
        RenderGraphExecutionRecord,
    pub(in crate::graphics::scene::scene_renderer::core) last_prepared_mesh_queue_stats:
        PreparedMeshQueueStats,
    pub(in crate::graphics::scene::scene_renderer::core) last_prepared_sprite_queue_stats:
        PreparedSpriteQueueStats,
    pub(in crate::graphics::scene::scene_renderer::core) frame_timing_report_requested: bool,
    pub(in crate::graphics::scene::scene_renderer::core) parallel_record_min_passes_per_bucket:
        Option<usize>,
    pub(in crate::graphics::scene::scene_renderer::core) hzb_diagnostics_readback_enabled: bool,
    pub(in crate::graphics::scene::scene_renderer::core) last_frame_timing_report:
        SceneRendererFrameTimingReport,
    pub(in crate::graphics::scene::scene_renderer::core) advanced_plugin_outputs:
        SceneRendererAdvancedPluginOutputs,
    // The backend owns the submission timeline and device generation, so it must drop last.
    pub(in crate::graphics::scene::scene_renderer::core) backend: RenderBackend,
}

/// One resolved GPU timestamp pass from a renderer frame.
///
/// This deliberately contains only framework-neutral timing data so tools do
/// not need access to the WGPU query/readback implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneRendererGpuPassTiming {
    pub(in crate::graphics::scene::scene_renderer::core) pass_name: String,
    pub(in crate::graphics::scene::scene_renderer::core) gpu_time_us: u64,
}

impl SceneRendererGpuPassTiming {
    pub fn new(pass_name: impl Into<String>, gpu_time_us: u64) -> Self {
        Self {
            pass_name: pass_name.into(),
            gpu_time_us,
        }
    }

    pub fn pass_name(&self) -> &str {
        &self.pass_name
    }

    pub const fn gpu_time_us(&self) -> u64 {
        self.gpu_time_us
    }
}

/// A completed, asynchronously resolved GPU timestamp frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneRendererGpuTimingReport {
    pub(in crate::graphics::scene::scene_renderer::core) frame_generation: u64,
    pub(in crate::graphics::scene::scene_renderer::core) timestamp_period_ns_bits: u32,
    pub(in crate::graphics::scene::scene_renderer::core) pass_timings:
        Vec<SceneRendererGpuPassTiming>,
    /// Frame-qualified submission counters attached only after the framework
    /// matches this resolved timestamp result to its retained frame profile.
    pub(in crate::graphics::scene::scene_renderer::core) mesh_submission:
        Option<RenderMeshSubmissionProfile>,
}

impl SceneRendererGpuTimingReport {
    pub fn new(
        frame_generation: u64,
        timestamp_period_ns: f32,
        pass_timings: impl IntoIterator<Item = SceneRendererGpuPassTiming>,
    ) -> Self {
        Self {
            frame_generation,
            timestamp_period_ns_bits: timestamp_period_ns.to_bits(),
            pass_timings: pass_timings.into_iter().collect(),
            mesh_submission: None,
        }
    }

    pub fn with_mesh_submission_profile(
        mut self,
        mesh_submission: RenderMeshSubmissionProfile,
    ) -> Self {
        self.mesh_submission = Some(mesh_submission);
        self
    }

    pub const fn frame_generation(&self) -> u64 {
        self.frame_generation
    }

    pub const fn timestamp_period_ns(&self) -> f32 {
        f32::from_bits(self.timestamp_period_ns_bits)
    }

    pub const fn timestamp_period_ns_bits(&self) -> u32 {
        self.timestamp_period_ns_bits
    }

    pub fn pass_timings(&self) -> &[SceneRendererGpuPassTiming] {
        &self.pass_timings
    }

    pub fn mesh_submission_profile(&self) -> Option<&RenderMeshSubmissionProfile> {
        self.mesh_submission.as_ref()
    }
}

#[derive(Clone, Copy)]
pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererCaptureTarget {
    pub(in crate::graphics::scene::scene_renderer::core) output_target:
        crate::graphics::types::ViewportRenderOutputTarget,
    pub(in crate::graphics::scene::scene_renderer::core) owns_final_target_output: bool,
}

/// Selects the built-in deferred-material set compiled during renderer startup.
///
/// `StandardPbrPreview` is for tools that render only opaque Standard PBR
/// materials. Consumers that can load arbitrary scenes must retain `FullScene`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SceneRendererDeferredLightingProfile {
    #[default]
    FullScene,
    StandardPbrPreview,
    /// Viewer-only Standard PBR path with IBL and emissive but no direct lights.
    EnvironmentOnlyPbrPreview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::core) enum ScenePostProcessStartupMode {
    Full,
    OutputTransferOnly,
}

impl SceneRendererDeferredLightingProfile {
    pub(in crate::graphics::scene::scene_renderer) const fn uses_gpu_scene(self) -> bool {
        !matches!(self, Self::EnvironmentOnlyPbrPreview)
    }

    pub(in crate::graphics::scene::scene_renderer) const fn uses_full_lighting_bind_group(
        self,
    ) -> bool {
        !matches!(self, Self::EnvironmentOnlyPbrPreview)
    }

    /// The fixed environment preview still uses GPU scene transforms, but its specialized
    /// shader has no direct-light consumer or light-buffer lookup.
    pub(in crate::graphics::scene::scene_renderer::core) const fn uses_direct_lights(self) -> bool {
        !matches!(self, Self::EnvironmentOnlyPbrPreview)
    }

    /// Particle, sprite and HZB resources are irrelevant to the fixed PBR viewer scene.
    pub(in crate::graphics::scene::scene_renderer) const fn uses_auxiliary_scene_effects(
        self,
    ) -> bool {
        !matches!(self, Self::EnvironmentOnlyPbrPreview)
    }

    /// A compiled scene graph can schedule arbitrary effects, while the fixed PBR viewer
    /// requires only the terminal HDR-to-output transfer.
    pub(in crate::graphics::scene::scene_renderer) const fn uses_full_post_process_resources(
        self,
    ) -> bool {
        self.supports_compiled_scene_graph()
    }

    /// Direct environment-preview rendering never builds a compiled scene graph.
    pub(in crate::graphics::scene::scene_renderer::core) const fn supports_compiled_scene_graph(
        self,
    ) -> bool {
        matches!(
            self.post_process_startup_mode(),
            ScenePostProcessStartupMode::Full
        )
    }

    pub(in crate::graphics::scene::scene_renderer::core) const fn post_process_startup_mode(
        self,
    ) -> ScenePostProcessStartupMode {
        match self {
            Self::FullScene | Self::StandardPbrPreview => ScenePostProcessStartupMode::Full,
            Self::EnvironmentOnlyPbrPreview => ScenePostProcessStartupMode::OutputTransferOnly,
        }
    }

    /// The PBR viewer submits no screen-space UI extract, so it does not need font or image UI
    /// systems during startup.
    pub(in crate::graphics::scene::scene_renderer) const fn uses_screen_space_ui(self) -> bool {
        !matches!(self, Self::EnvironmentOnlyPbrPreview)
    }

    pub(in crate::graphics::scene::scene_renderer::core) const fn defers_local_reflection_provider_resources(
        self,
    ) -> bool {
        matches!(self, Self::EnvironmentOnlyPbrPreview)
    }

    pub(in crate::graphics::scene::scene_renderer::core) const fn uses_full_shadow_atlas_resources(
        self,
    ) -> bool {
        !matches!(self, Self::EnvironmentOnlyPbrPreview)
    }

    /// Shadow-map recording exists only in the compiled scene graph, which the fixed viewer
    /// never executes. EnvironmentOnly retains the lightweight atlas binding placeholder.
    pub(in crate::graphics::scene::scene_renderer::core) const fn uses_shadow_map_renderer(
        self,
    ) -> bool {
        self.uses_full_shadow_atlas_resources()
    }

    /// Selection, grid, gizmo, and transform handles are editor interaction affordances.
    /// The fixed PBR viewer never extracts them, so it does not construct or record them.
    pub(in crate::graphics::scene::scene_renderer::core) const fn uses_interaction_overlays(
        self,
    ) -> bool {
        !matches!(self, Self::EnvironmentOnlyPbrPreview)
    }
}

/// Startup-only renderer options. The default preserves the full scene renderer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SceneRendererStartupOptions {
    deferred_lighting_profile: SceneRendererDeferredLightingProfile,
    allow_gpu_timing: bool,
    async_pipeline_compile: bool,
    environment_only_base_prewarm: bool,
}

impl SceneRendererStartupOptions {
    pub const fn standard_pbr_preview() -> Self {
        Self {
            deferred_lighting_profile: SceneRendererDeferredLightingProfile::StandardPbrPreview,
            allow_gpu_timing: false,
            async_pipeline_compile: false,
            environment_only_base_prewarm: false,
        }
    }

    /// Uses the smallest deferred contract for the zero-direct-light PBR viewer.
    pub const fn environment_only_pbr_preview() -> Self {
        Self {
            deferred_lighting_profile:
                SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
            allow_gpu_timing: false,
            async_pipeline_compile: false,
            environment_only_base_prewarm: true,
        }
    }

    /// Opts startup into timestamp-query resources; the default allocates none.
    pub const fn with_gpu_timing(mut self) -> Self {
        self.allow_gpu_timing = true;
        self
    }

    /// Queues Base-pass PSO creation so an interactive host can present while it compiles.
    pub const fn with_async_pipeline_compile(mut self) -> Self {
        self.async_pipeline_compile = true;
        self
    }

    /// Retains the light-weight environment-only renderer contract while leaving
    /// its specialized Base PSO to an explicit owner. This is used when a
    /// diagnostic fixture submits a different generic Forward material variant.
    pub const fn without_environment_only_pbr_base_prewarm(mut self) -> Self {
        self.environment_only_base_prewarm = false;
        self
    }

    pub(in crate::graphics::scene::scene_renderer::core) const fn allow_gpu_timing(self) -> bool {
        self.allow_gpu_timing
    }

    pub(in crate::graphics::scene::scene_renderer::core) const fn async_pipeline_compile_enabled(
        self,
    ) -> bool {
        self.async_pipeline_compile
    }

    pub const fn deferred_lighting_profile(self) -> SceneRendererDeferredLightingProfile {
        self.deferred_lighting_profile
    }

    /// The environment-only viewer owns a synchronous warmup for its first Base PSO unless its
    /// host selects asynchronous pipeline compilation.
    pub(in crate::graphics::scene::scene_renderer::core) const fn requires_environment_only_pbr_base_prewarm(
        self,
    ) -> bool {
        matches!(
            self.deferred_lighting_profile,
            SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
        ) && self.environment_only_base_prewarm
            && !self.async_pipeline_compile
    }

    pub(in crate::graphics::scene::scene_renderer::core) const fn queues_environment_only_pbr_base_prewarm(
        self,
    ) -> bool {
        matches!(
            self.deferred_lighting_profile,
            SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
        ) && self.environment_only_base_prewarm
            && self.async_pipeline_compile
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SceneRendererFrameTimingReport {
    pub(in crate::graphics::scene::scene_renderer::core) render_submission: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) readback_and_completion: Duration,
}

impl SceneRendererFrameTimingReport {
    pub const fn render_submission(self) -> Duration {
        self.render_submission
    }

    pub const fn readback_and_completion(self) -> Duration {
        self.readback_and_completion
    }
}

#[cfg(test)]
#[path = "tests/scene_renderer.rs"]
mod tests;

impl SceneRenderer {
    pub(crate) fn prepared_local_bounds_for_mesh(
        &self,
        mesh: &RenderMeshSnapshot,
    ) -> Option<RenderMeshBounds> {
        self.streamer
            .render_mesh_geometry_seed(mesh)
            .and_then(local_bounds_for_cpu_visibility)
    }

    /// Shares the backend-owned gate with the framework submission owner.
    pub(in crate::graphics) fn device_fault_gate(&self) -> std::sync::Arc<zr_rhi::DeviceFaultGate> {
        self.backend.device_fault_gate()
    }

    pub(crate) fn next_frame_generation(&self) -> u64 {
        self.generation.wrapping_add(1)
    }

    pub(crate) fn supports_compiled_scene_graph(&self) -> bool {
        self.core
            .deferred_lighting_profile
            .supports_compiled_scene_graph()
    }

    pub(crate) fn set_global_material_mip_bias(&mut self, mip_bias: f32) {
        self.core.global_material_mip_bias = mip_bias;
    }

    pub fn request_next_frame_timing_report(&mut self) {
        self.frame_timing_report_requested = true;
    }

    pub fn last_frame_timing_report(&self) -> SceneRendererFrameTimingReport {
        self.last_frame_timing_report
    }

    /// Reports whether the environment-only viewer's queued Base PSO can draw.
    ///
    /// A `false` result means the host should keep its event loop responsive and
    /// request another frame. A terminal background compilation error is returned
    /// instead of silently capturing a deferred Base draw admission.
    pub fn environment_only_pbr_base_pipeline_ready(&mut self) -> Result<bool, GraphicsError> {
        self.core
            .mesh_pipelines
            .environment_only_pbr_base_pipeline_ready()
    }

    /// Retries admission of the viewer's required Base PSO without blocking.
    pub fn retry_environment_only_pbr_base_pipeline_admission(
        &mut self,
    ) -> Result<(), GraphicsError> {
        self.core
            .mesh_pipelines
            .queue_environment_only_pbr_base_pipeline(&self.backend.device, &mut self.streamer)?;
        Ok(())
    }

    /// Reports the generic Forward Base PSO required by a non-default-IOR
    /// material after it has been explicitly queued by its diagnostic owner.
    pub fn pbr_ior_forward_base_pipeline_ready(&mut self) -> Result<bool, GraphicsError> {
        self.core
            .mesh_pipelines
            .pbr_ior_forward_base_pipeline_ready()
    }

    /// Queues nonblocking admission for the static non-default-IOR generic
    /// Forward Base PSO. Repeated calls retry after bounded compiler backpressure.
    pub fn queue_pbr_ior_forward_base_pipeline_admission(&mut self) -> Result<(), GraphicsError> {
        self.core
            .mesh_pipelines
            .queue_pbr_ior_forward_base_pipeline(&self.backend.device, &mut self.streamer)
    }

    pub(in crate::graphics) fn set_parallel_recording(
        &mut self,
        enabled: bool,
        min_passes_per_bucket: usize,
    ) {
        self.parallel_record_min_passes_per_bucket =
            enabled.then_some(min_passes_per_bucket.max(1));
    }

    pub(in crate::graphics) fn set_hzb_diagnostics_readback_enabled(&mut self, enabled: bool) {
        self.hzb_diagnostics_readback_enabled = enabled;
    }

    #[cfg(test)]
    pub(in crate::graphics) const fn hzb_diagnostics_readback_enabled(&self) -> bool {
        self.hzb_diagnostics_readback_enabled
    }

    #[cfg(test)]
    pub(in crate::graphics) fn parallel_record_min_passes_per_bucket(&self) -> Option<usize> {
        self.parallel_record_min_passes_per_bucket
    }
}

fn local_bounds_for_cpu_visibility(
    seed: RuntimePrepareMeshGeometrySeed,
) -> Option<RenderMeshBounds> {
    if matches!(
        &seed.mesh_sdf,
        RuntimePrepareMeshSdfSeed::Deforming(RuntimePrepareMeshSdfDeformationReason::Skinning)
    ) {
        None
    } else {
        Some(seed.local_bounds)
    }
}
