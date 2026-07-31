use std::collections::HashMap;
use std::time::Duration;

use crate::core::framework::render::FrameHistoryHandle;

use super::super::scene_renderer_core::SceneRendererCore;
use crate::graphics::backend::{GpuPassTimer, GpuTimerFrameResult, OffscreenTarget, RenderBackend};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::mesh::{
    EnvironmentOnlyPbrBasePipelinePrewarmReport, PreparedMeshQueueStats,
};
use crate::graphics::scene::scene_renderer::sprite::PreparedSpriteQueueStats;

use super::super::super::graph_execution::{
    RenderGraphExecutionRecord, RenderPassExecutorRegistry,
};
use super::advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;

pub struct SceneRenderer {
    pub(in crate::graphics::scene::scene_renderer::core) backend: RenderBackend,
    pub(in crate::graphics::scene::scene_renderer::core) core: SceneRendererCore,
    pub(in crate::graphics::scene::scene_renderer::core) streamer: ResourceStreamer,
    pub(in crate::graphics::scene::scene_renderer::core) target: Option<OffscreenTarget>,
    pub(in crate::graphics::scene::scene_renderer::core) history_targets:
        HashMap<FrameHistoryHandle, SceneFrameHistoryTextures>,
    pub(in crate::graphics::scene::scene_renderer::core) generation: u64,
    pub(in crate::graphics::scene::scene_renderer::core) gpu_pass_timer: Option<GpuPassTimer>,
    pub(in crate::graphics::scene::scene_renderer::core) last_gpu_timer_frame_result:
        Option<GpuTimerFrameResult>,
    pub(in crate::graphics::scene::scene_renderer::core) render_pass_executors:
        RenderPassExecutorRegistry,
    pub(in crate::graphics::scene::scene_renderer::core) last_render_graph_execution:
        RenderGraphExecutionRecord,
    pub(in crate::graphics::scene::scene_renderer::core) last_prepared_mesh_queue_stats:
        PreparedMeshQueueStats,
    pub(in crate::graphics::scene::scene_renderer::core) last_prepared_sprite_queue_stats:
        PreparedSpriteQueueStats,
    pub(in crate::graphics::scene::scene_renderer::core) frame_timing_report_requested: bool,
    pub(in crate::graphics::scene::scene_renderer::core) last_frame_timing_report:
        SceneRendererFrameTimingReport,
    pub(in crate::graphics::scene::scene_renderer::core) advanced_plugin_outputs:
        SceneRendererAdvancedPluginOutputs,
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

impl SceneRendererDeferredLightingProfile {
    pub(in crate::graphics::scene::scene_renderer) const fn uses_gpu_scene(self) -> bool {
        !matches!(self, Self::EnvironmentOnlyPbrPreview)
    }

    pub(in crate::graphics::scene::scene_renderer) const fn uses_full_lighting_bind_group(
        self,
    ) -> bool {
        !matches!(self, Self::EnvironmentOnlyPbrPreview)
    }
}

/// Startup-only renderer options. The default preserves the full scene renderer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SceneRendererStartupOptions {
    deferred_lighting_profile: SceneRendererDeferredLightingProfile,
}

impl SceneRendererStartupOptions {
    pub const fn standard_pbr_preview() -> Self {
        Self {
            deferred_lighting_profile: SceneRendererDeferredLightingProfile::StandardPbrPreview,
        }
    }

    /// Uses the smallest deferred contract for the zero-direct-light PBR viewer.
    pub const fn environment_only_pbr_preview() -> Self {
        Self {
            deferred_lighting_profile:
                SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
        }
    }

    pub const fn deferred_lighting_profile(self) -> SceneRendererDeferredLightingProfile {
        self.deferred_lighting_profile
    }

    /// The environment-only viewer owns a synchronous warmup for its first Base PSO.
    pub(in crate::graphics::scene::scene_renderer::core) const fn requires_environment_only_pbr_base_prewarm(
        self,
    ) -> bool {
        matches!(
            self.deferred_lighting_profile,
            SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
        )
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

/// Component timings for the environment-only viewer's synchronous Base PSO warmup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneRendererEnvironmentOnlyPbrBasePrewarmReport {
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
            cache_hit: report.cache_hit(),
            shader_source_resolution: report.shader_source_resolution(),
            pipeline_creation: report.pipeline_creation(),
            elapsed: report.elapsed(),
        }
    }
}

impl SceneRendererEnvironmentOnlyPbrBasePrewarmReport {
    pub const fn cache_hit(self) -> bool {
        self.cache_hit
    }

    pub const fn shader_source_resolution(self) -> Duration {
        self.shader_source_resolution
    }

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

    /// The startup cost paid to prepare the exact environment-only PBR Base PSO.
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
    use super::SceneRendererStartupOptions;

    #[test]
    fn environment_only_pbr_profile_is_the_only_startup_profile_that_prewarm_base() {
        assert!(
            !SceneRendererStartupOptions::default().requires_environment_only_pbr_base_prewarm()
        );
        assert!(
            !SceneRendererStartupOptions::standard_pbr_preview()
                .requires_environment_only_pbr_base_prewarm()
        );
        assert!(
            SceneRendererStartupOptions::environment_only_pbr_preview()
                .requires_environment_only_pbr_base_prewarm()
        );
    }
}

impl SceneRenderer {
    pub fn request_next_frame_timing_report(&mut self) {
        self.frame_timing_report_requested = true;
    }

    pub fn last_frame_timing_report(&self) -> SceneRendererFrameTimingReport {
        self.last_frame_timing_report
    }
}
