use std::time::Duration;

use crate::graphics::scene::scene_renderer::mesh::EnvironmentOnlyPbrBasePipelinePrewarmReport;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneRendererStartupReport {
    pub(in crate::graphics::scene::scene_renderer::core) backend_initialization: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) system_texture_initialization: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) system_texture_builtin_payload_materialized:
        bool,
    pub(in crate::graphics::scene::scene_renderer::core) system_texture_builtin_payload_cache_wait:
        Duration,
    pub(in crate::graphics::scene::scene_renderer::core) system_texture_builtin_payload_materialization:
        Duration,
    pub(in crate::graphics::scene::scene_renderer::core) system_texture_upload_submission: Duration,
    pub(in crate::graphics::scene::scene_renderer::core) system_texture_upload_ticket:
        Option<zr_rhi::SubmissionTicket>,
    pub(in crate::graphics::scene::scene_renderer::core) system_texture_upload_count: usize,
    pub(in crate::graphics::scene::scene_renderer::core) system_texture_upload_bytes: u64,
    pub(in crate::graphics::scene::scene_renderer::core) system_texture_native_submission_count:
        usize,
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

impl SceneRendererStartupReport {
    pub const fn backend_initialization(self) -> Duration {
        self.backend_initialization
    }

    pub const fn system_texture_initialization(self) -> Duration {
        self.system_texture_initialization
    }

    pub const fn system_texture_builtin_payload_materialized(self) -> bool {
        self.system_texture_builtin_payload_materialized
    }

    pub const fn system_texture_builtin_payload_cache_wait(self) -> Duration {
        self.system_texture_builtin_payload_cache_wait
    }

    pub const fn system_texture_builtin_payload_materialization(self) -> Duration {
        self.system_texture_builtin_payload_materialization
    }

    /// CPU time to admit and flush the generation's immutable texture packet.
    pub const fn system_texture_upload_submission(self) -> Duration {
        self.system_texture_upload_submission
    }

    pub const fn system_texture_upload_ticket(self) -> Option<zr_rhi::SubmissionTicket> {
        self.system_texture_upload_ticket
    }

    pub const fn system_texture_upload_count(self) -> usize {
        self.system_texture_upload_count
    }

    pub const fn system_texture_upload_bytes(self) -> u64 {
        self.system_texture_upload_bytes
    }

    pub const fn system_texture_native_submission_count(self) -> usize {
        self.system_texture_native_submission_count
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
