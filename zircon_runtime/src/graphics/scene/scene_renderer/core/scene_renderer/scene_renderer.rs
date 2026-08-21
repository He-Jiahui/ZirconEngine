use std::collections::HashMap;
use std::time::Duration;

use crate::core::framework::render::{FrameHistoryHandle, RenderGpuTimingStatus};

use super::super::scene_renderer_core::SceneRendererCore;
use crate::graphics::backend::{
    GpuPassTimer, GpuPipelineStatisticsFrameResult, GpuPipelineStatisticsTimer,
    GpuTimerFrameResult, OffscreenTarget, RenderBackend,
};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::mesh::{
    EnvironmentOnlyPbrBasePipelinePrewarmReport, PreparedMeshQueueStats,
};
use crate::graphics::scene::scene_renderer::sprite::PreparedSpriteQueueStats;
use crate::graphics::types::GraphicsError;

use super::super::super::graph_execution::{
    RenderGraphExecutionRecord, RenderPassExecutorRegistry,
};
use super::advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;

pub struct SceneRenderer {
    pub(in crate::graphics::scene::scene_renderer::core) backend: RenderBackend,
    pub(in crate::graphics::scene::scene_renderer::core) core: SceneRendererCore,
    pub(in crate::graphics::scene::scene_renderer::core) streamer: ResourceStreamer,
    pub(in crate::graphics::scene::scene_renderer::core) target: Option<OffscreenTarget>,
    pub(in crate::graphics::scene::scene_renderer::core) last_capture_target:
        Option<SceneRendererCaptureTarget>,
    pub(in crate::graphics::scene::scene_renderer::core) history_targets:
        HashMap<FrameHistoryHandle, SceneFrameHistoryTextures>,
    pub(in crate::graphics::scene::scene_renderer::core) generation: u64,
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
        }
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
}

impl SceneRendererStartupOptions {
    pub const fn standard_pbr_preview() -> Self {
        Self {
            deferred_lighting_profile: SceneRendererDeferredLightingProfile::StandardPbrPreview,
            allow_gpu_timing: false,
            async_pipeline_compile: false,
        }
    }

    /// Uses the smallest deferred contract for the zero-direct-light PBR viewer.
    pub const fn environment_only_pbr_preview() -> Self {
        Self {
            deferred_lighting_profile:
                SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
            allow_gpu_timing: false,
            async_pipeline_compile: false,
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
        ) && !self.async_pipeline_compile
    }

    pub(in crate::graphics::scene::scene_renderer::core) const fn queues_environment_only_pbr_base_prewarm(
        self,
    ) -> bool {
        matches!(
            self.deferred_lighting_profile,
            SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
        ) && self.async_pipeline_compile
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneRendererStartupReport {
    pub(in crate::graphics::scene::scene_renderer::core) backend_initialization: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) core_initialization: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) core_startup:
        SceneRendererCoreStartupReport,
    pub(in crate::graphics::scene::scene_renderer::core) resource_streamer_initialization: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) environment_only_pbr_base_prewarm:
        Option<SceneRendererEnvironmentOnlyPbrBasePrewarmReport>,
}

/// Component timings for the environment-only viewer's Base PSO warmup request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneRendererEnvironmentOnlyPbrBasePrewarmReport {
    pipeline_ready: bool,
    cache_hit: bool,
    shader_source_resolution: Duration,
    pipeline_creation: Duration,
    elapsed: Duration,
}

impl From<EnvironmentOnlyPbrBasePipelinePrewarmReport>
    for SceneRendererEnvironmentOnlyPbrBasePrewarmReport
{
    fn from(report: EnvironmentOnlyPbrBasePipelinePrewarmReport) -> Self {
        Self {
            pipeline_ready: report.pipeline_ready(),
            cache_hit: report.cache_hit(),
            shader_source_resolution: report.shader_source_resolution(),
            pipeline_creation: report.pipeline_creation(),
            elapsed: report.elapsed(),
        }
    }
}

impl SceneRendererEnvironmentOnlyPbrBasePrewarmReport {
    /// Whether the Base PSO was ready before renderer construction returned.
    pub const fn pipeline_ready(self) -> bool {
        self.pipeline_ready
    }

    pub const fn cache_hit(self) -> bool {
        self.cache_hit
    }

    pub const fn shader_source_resolution(self) -> Duration {
        self.shader_source_resolution
    }

    /// Synchronous creation time, or background queue-submission time when `pipeline_ready` is
    /// false.
    pub const fn pipeline_creation(self) -> Duration {
        self.pipeline_creation
    }

    pub const fn elapsed(self) -> Duration {
        self.elapsed
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneRendererCoreStartupReport {
    pub(in crate::graphics::scene::scene_renderer::core) setup: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) mesh_and_environment: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) shadows: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) deferred: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) deferred_lighting_pipelines: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) deferred_lighting_shader_source_assembly:
        Duration,
    pub(in crate::graphics::scene::scene_renderer::core) deferred_lighting_pipeline_foundation:
        Duration,
    pub(in crate::graphics::scene::scene_renderer::core) deferred_lighting_standard_pipeline:
        Duration,
    pub(in crate::graphics::scene::scene_renderer::core) deferred_fallback_resources: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) scene_effects: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) scene_effects_particles: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) scene_effects_sprites: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) scene_effects_hzb: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) scene_effects_post_process: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) overlay_and_ui: Duration,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SceneRendererFrameTimingReport {
    pub(in crate::graphics::scene::scene_renderer::core) render_submission: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) readback_and_completion: Duration,
}

impl SceneRendererStartupReport {
    pub const fn backend_initialization(self) -> Duration {
        self.backend_initialization
    }

    pub const fn core_initialization(self) -> Duration {
        self.core_initialization
    }

    pub const fn core_startup(self) -> SceneRendererCoreStartupReport {
        self.core_startup
    }

    pub const fn resource_streamer_initialization(self) -> Duration {
        self.resource_streamer_initialization
    }

    /// The startup cost to prepare or queue the exact environment-only PBR Base PSO.
    pub const fn environment_only_pbr_base_prewarm(
        self,
    ) -> Option<SceneRendererEnvironmentOnlyPbrBasePrewarmReport> {
        self.environment_only_pbr_base_prewarm
    }
}

impl SceneRendererFrameTimingReport {
    pub const fn render_submission(self) -> Duration {
        self.render_submission
    }

    pub const fn readback_and_completion(self) -> Duration {
        self.readback_and_completion
    }
}

impl SceneRendererCoreStartupReport {
    pub const fn setup(self) -> Duration {
        self.setup
    }

    pub const fn mesh_and_environment(self) -> Duration {
        self.mesh_and_environment
    }

    pub const fn shadows(self) -> Duration {
        self.shadows
    }

    pub const fn deferred(self) -> Duration {
        self.deferred
    }

    pub const fn deferred_lighting_pipelines(self) -> Duration {
        self.deferred_lighting_pipelines
    }

    pub const fn deferred_lighting_shader_source_assembly(self) -> Duration {
        self.deferred_lighting_shader_source_assembly
    }

    pub const fn deferred_lighting_pipeline_foundation(self) -> Duration {
        self.deferred_lighting_pipeline_foundation
    }

    pub const fn deferred_lighting_standard_pipeline(self) -> Duration {
        self.deferred_lighting_standard_pipeline
    }

    pub const fn deferred_fallback_resources(self) -> Duration {
        self.deferred_fallback_resources
    }

    pub const fn scene_effects(self) -> Duration {
        self.scene_effects
    }

    pub const fn scene_effects_particles(self) -> Duration {
        self.scene_effects_particles
    }

    pub const fn scene_effects_sprites(self) -> Duration {
        self.scene_effects_sprites
    }

    pub const fn scene_effects_hzb(self) -> Duration {
        self.scene_effects_hzb
    }

    pub const fn scene_effects_post_process(self) -> Duration {
        self.scene_effects_post_process
    }

    pub const fn overlay_and_ui(self) -> Duration {
        self.overlay_and_ui
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::asset::{ProjectAssetManager, ProjectAssetManagerAccess};

    use super::{
        ScenePostProcessStartupMode, SceneRenderer, SceneRendererDeferredLightingProfile,
        SceneRendererStartupOptions,
    };

    #[test]
    fn environment_only_pbr_profile_is_the_only_startup_profile_that_prewarm_base() {
        assert!(
            !SceneRendererStartupOptions::default().requires_environment_only_pbr_base_prewarm()
        );
        assert!(!SceneRendererStartupOptions::standard_pbr_preview()
            .requires_environment_only_pbr_base_prewarm());
        assert!(SceneRendererStartupOptions::environment_only_pbr_preview()
            .requires_environment_only_pbr_base_prewarm());
    }

    #[test]
    fn environment_only_async_pipeline_startup_queues_base_instead_of_waiting_for_it() {
        let options = SceneRendererStartupOptions::environment_only_pbr_preview()
            .with_async_pipeline_compile();

        assert!(options.async_pipeline_compile_enabled());
        assert!(!options.requires_environment_only_pbr_base_prewarm());
        assert!(options.queues_environment_only_pbr_base_prewarm());
    }

    #[test]
    fn environment_only_pbr_preview_omits_auxiliary_scene_effects() {
        assert!(SceneRendererDeferredLightingProfile::FullScene.uses_auxiliary_scene_effects());
        assert!(
            SceneRendererDeferredLightingProfile::StandardPbrPreview.uses_auxiliary_scene_effects()
        );
        assert!(
            !SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
                .uses_auxiliary_scene_effects()
        );
        assert!(SceneRendererDeferredLightingProfile::FullScene.uses_full_post_process_resources());
        assert!(SceneRendererDeferredLightingProfile::StandardPbrPreview
            .uses_full_post_process_resources());
        assert!(
            !SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
                .uses_full_post_process_resources()
        );
        assert!(SceneRendererDeferredLightingProfile::FullScene.supports_compiled_scene_graph());
        assert!(SceneRendererDeferredLightingProfile::StandardPbrPreview
            .supports_compiled_scene_graph());
        assert!(
            !SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
                .supports_compiled_scene_graph()
        );
        assert!(!SceneRendererDeferredLightingProfile::FullScene
            .defers_local_reflection_provider_resources());
        assert!(!SceneRendererDeferredLightingProfile::StandardPbrPreview
            .defers_local_reflection_provider_resources());
        assert!(
            SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
                .defers_local_reflection_provider_resources()
        );
        assert!(SceneRendererDeferredLightingProfile::FullScene.uses_screen_space_ui());
        assert!(SceneRendererDeferredLightingProfile::StandardPbrPreview.uses_screen_space_ui());
        assert!(
            !SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview.uses_screen_space_ui()
        );
    }

    #[test]
    fn environment_only_pbr_preview_uses_a_shadow_atlas_placeholder() {
        assert!(SceneRendererDeferredLightingProfile::FullScene.uses_full_shadow_atlas_resources());
        assert!(SceneRendererDeferredLightingProfile::StandardPbrPreview
            .uses_full_shadow_atlas_resources());
        assert!(
            !SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
                .uses_full_shadow_atlas_resources()
        );
    }

    #[test]
    fn environment_only_pbr_preview_omits_direct_light_buffer_work() {
        assert!(SceneRendererDeferredLightingProfile::FullScene.uses_direct_lights());
        assert!(SceneRendererDeferredLightingProfile::StandardPbrPreview.uses_direct_lights());
        assert!(
            !SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview.uses_direct_lights()
        );
    }

    #[test]
    fn environment_only_pbr_shader_has_no_direct_light_buffer_lookup() {
        let source = include_str!("../../../../shader/wgsl/zr_shading_environment_only_pbr.wgsl");

        assert!(
            !source.contains("zr_gpu_scene_light"),
            "the environment-only shader must not consume the direct-light buffer"
        );
    }

    #[test]
    fn environment_only_pbr_preview_omits_compiled_scene_shadow_renderer() {
        assert!(SceneRendererDeferredLightingProfile::FullScene.uses_shadow_map_renderer());
        assert!(SceneRendererDeferredLightingProfile::StandardPbrPreview.uses_shadow_map_renderer());
        assert!(
            !SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
                .uses_shadow_map_renderer()
        );
    }

    #[test]
    fn shadow_map_renderer_construction_tracks_deferred_lighting_profile() {
        let asset_manager = Arc::new(ProjectAssetManager::default());
        for (profile_name, startup_options, expects_shadow_map_renderer) in [
            ("full-scene", SceneRendererStartupOptions::default(), true),
            (
                "standard-pbr-preview",
                SceneRendererStartupOptions::standard_pbr_preview(),
                true,
            ),
            (
                "environment-only-pbr-preview",
                SceneRendererStartupOptions::environment_only_pbr_preview(),
                false,
            ),
        ] {
            let (renderer, _) = SceneRenderer::new_with_startup_options_and_report(
                ProjectAssetManagerAccess::for_test(Arc::clone(&asset_manager)),
                startup_options,
            )
            .unwrap_or_else(|error| panic!("{profile_name} renderer startup failed: {error}"));

            assert_eq!(
                renderer.core.shadow_map_renderer.is_some(),
                expects_shadow_map_renderer,
                "{profile_name} shadow renderer construction did not match its profile"
            );
        }
    }

    #[test]
    fn environment_only_pbr_preview_excludes_editor_interaction_overlays() {
        assert!(SceneRendererDeferredLightingProfile::FullScene.uses_interaction_overlays());
        assert!(
            SceneRendererDeferredLightingProfile::StandardPbrPreview.uses_interaction_overlays()
        );
        assert!(
            !SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
                .uses_interaction_overlays()
        );
    }

    #[test]
    fn post_process_startup_mode_preserves_full_graphs_and_selects_viewer_transfer_only() {
        assert_eq!(
            SceneRendererDeferredLightingProfile::FullScene.post_process_startup_mode(),
            ScenePostProcessStartupMode::Full
        );
        assert_eq!(
            SceneRendererDeferredLightingProfile::StandardPbrPreview.post_process_startup_mode(),
            ScenePostProcessStartupMode::Full
        );
        assert_eq!(
            SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
                .post_process_startup_mode(),
            ScenePostProcessStartupMode::OutputTransferOnly
        );
    }

    #[test]
    fn gpu_timing_resources_require_an_explicit_startup_option() {
        assert!(!SceneRendererStartupOptions::default().allow_gpu_timing());
        assert!(SceneRendererStartupOptions::default()
            .with_gpu_timing()
            .allow_gpu_timing());
    }
}

impl SceneRenderer {
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
    /// instead of silently capturing the `SkipDraw` placeholder.
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
