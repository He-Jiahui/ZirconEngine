use crate::core::framework::render::{RenderCapabilitySummary, RenderFrameSubmissionTransaction};
use crate::core::TaskPool;
use crate::graphics::backend::{
    GpuPassTimer, GpuPipelineStatisticsTimer, OffscreenTarget, RenderBackend, ViewportSurface,
};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::core::scene_renderer_render_with_pipeline::AsyncViewportCaptureRequest;
use crate::graphics::scene::scene_renderer::graph_execution::RenderPassExecutorRegistry;
use crate::graphics::scene::scene_renderer::history::{
    SceneFrameHistoryTextures, SceneHistoryFrameTransaction,
};
use crate::graphics::scene::scene_renderer::post_process::SceneRuntimeFeatureFlags;
use crate::graphics::types::ViewportRenderFrame;
use crate::graphics::{CompiledRenderPipeline, EnvironmentIblBakeReservation};

use super::super::super::scene_renderer_core::SceneRendererCore;
use super::super::SceneRendererCompiledSceneOutputs;

use crate::graphics::scene::scene_renderer::environment::{
    RealtimeIblPendingSubmission, RealtimeIblRuntime,
};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, TransientResourcePool,
};
use crate::graphics::scene::scene_renderer::SceneRendererDeferredLightingProfile;
use crate::graphics::types::GraphicsError;

pub(super) fn ensure_compiled_scene_graph_resources(
    deferred_lighting_profile: SceneRendererDeferredLightingProfile,
    has_full_post_process_resources: bool,
    has_scene_clear_resources: bool,
) -> Result<(), GraphicsError> {
    if !deferred_lighting_profile.supports_compiled_scene_graph() {
        return Err(GraphicsError::Asset(
            "environment-only PBR startup profile cannot execute a compiled scene graph".to_owned(),
        ));
    }
    if !has_full_post_process_resources {
        return Err(GraphicsError::Asset(
            "compiled scene graph requires full post-process resources".to_owned(),
        ));
    }
    if !has_scene_clear_resources {
        return Err(GraphicsError::Asset(
            "compiled scene graph requires scene-clear resources".to_owned(),
        ));
    }
    Ok(())
}

/// Returns transient backings after an error leaves an unsubmitted graph frame.
pub(super) fn abort_compiled_scene_graph_resource_frame(
    graph_resources: &mut RenderGraphExecutionResources,
    transient_resource_pool: &mut TransientResourcePool,
) {
    graph_resources.release_transient_backings_into_pool(transient_resource_pool);
    transient_resource_pool.end_frame();
}

pub(super) fn abort_realtime_ibl_submission(
    realtime_ibl: &mut RealtimeIblRuntime,
    submission: &mut Option<RealtimeIblPendingSubmission>,
) {
    if let Some(submission) = submission.take() {
        realtime_ibl.complete_submission(submission, false);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RenderGenerationIds {
    frame: u64,
    pub(super) mesh_commands: u64,
}

impl RenderGenerationIds {
    pub(super) fn new(frame: u64, mesh_commands: u64) -> Self {
        Self {
            frame,
            mesh_commands,
        }
    }

    pub(super) fn timer_frame(self) -> u64 {
        self.frame
    }
}

impl SceneRendererCore {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_compiled_scene(
        &mut self,
        backend: &RenderBackend,
        streamer: &mut ResourceStreamer,
        frame: &ViewportRenderFrame,
        target: &mut OffscreenTarget,
        pipeline: &CompiledRenderPipeline,
        capabilities: &RenderCapabilitySummary,
        render_pass_executors: &RenderPassExecutorRegistry,
        runtime_features: SceneRuntimeFeatureFlags,
        history_textures: Option<&mut SceneFrameHistoryTextures>,
        history_frame_transaction: SceneHistoryFrameTransaction,
        history_initialization_command_buffer: Option<wgpu::CommandBuffer>,
        frame_generation: u64,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
        gpu_pass_timer: Option<&mut GpuPassTimer>,
        gpu_pipeline_statistics_timer: Option<&mut GpuPipelineStatisticsTimer>,
        compute_task_pool: Option<&TaskPool>,
        parallel_record_min_passes_per_bucket: Option<usize>,
        hzb_diagnostics_readback_enabled: bool,
        viewport_capture: Option<AsyncViewportCaptureRequest>,
        environment_ibl_bake_reservation: Option<EnvironmentIblBakeReservation>,
        viewport_product_copy: Option<&zr_rhi_wgpu::WgpuUiExternalImageCopyTarget>,
        surface_frame: Option<(&ViewportSurface, &zr_rhi_wgpu::WgpuNativeSurfaceFrameTarget)>,
    ) -> Result<SceneRendererCompiledSceneOutputs, GraphicsError> {
        let mut environment_frame = self.begin_scene_environment_frame();
        environment_frame
            .core()
            .render_compiled_scene_in_environment_frame(
                backend,
                streamer,
                frame,
                target,
                pipeline,
                capabilities,
                render_pass_executors,
                runtime_features,
                history_textures,
                history_frame_transaction,
                history_initialization_command_buffer,
                frame_generation,
                submission_transaction,
                gpu_pass_timer,
                gpu_pipeline_statistics_timer,
                compute_task_pool,
                parallel_record_min_passes_per_bucket,
                hzb_diagnostics_readback_enabled,
                viewport_capture,
                environment_ibl_bake_reservation,
                viewport_product_copy,
                surface_frame,
            )
    }
}
