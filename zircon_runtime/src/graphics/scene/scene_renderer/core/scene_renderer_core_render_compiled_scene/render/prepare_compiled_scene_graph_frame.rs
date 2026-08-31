use crate::core::framework::render::{
    AntiAliasMode, PostProcessGraphResourceNames, RenderPluginRendererOutputs,
};
use crate::graphics::CompiledRenderPipeline;
use crate::graphics::backend::{
    GpuPassTimer, GpuPipelineStatisticsTimer, OffscreenTarget, ProductDiagnosticQueryFrameScope,
    ProductDiagnosticReadbackFrameScope, RenderBackend,
};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::environment::RealtimeIblPendingSubmission;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionRecord, RenderGraphExecutionResources, RenderPassMeshCommandLists,
};
use crate::graphics::scene::scene_renderer::history::{
    SceneFrameHistoryTextures, SceneHistoryAvailability,
};
use crate::graphics::scene::scene_renderer::post_process::SceneRuntimeFeatureFlags;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use zr_rhi_wgpu::WgpuBufferUploadBatch;

use super::super::super::scene_renderer_core::{
    SceneRendererAdvancedPluginReadbacks, SceneRendererCore,
};
use super::bind_compiled_scene_graph_resources::{
    CompiledSceneGraphResourceBindingFlags, bind_compiled_scene_graph_resources,
};
use super::final_target_output::FinalTargetOutputSelection;
use super::frame_lifecycle::{
    RenderGenerationIds, abort_compiled_scene_graph_resource_frame, abort_realtime_ibl_submission,
};
use super::pipeline_resource_usage::pipeline_writes_resource;

pub(super) struct PreparedCompiledSceneGraphFrame<'a> {
    pub(super) material_gbuffer_valid: bool,
    pub(super) taa_history_enabled: bool,
    pub(super) screen_space_reflection_history_enabled: bool,
    pub(super) hzb_history_enabled: bool,
    pub(super) exposure_history_enabled: bool,
    pub(super) exposure_history_reset_prepared: bool,
    pub(super) volumetric_history_enabled: bool,
    pub(super) product_diagnostic_frame_scope: Option<ProductDiagnosticReadbackFrameScope<'a>>,
    pub(super) product_diagnostic_query_scope: Option<ProductDiagnosticQueryFrameScope<'a>>,
    pub(super) diagnostic_frame_index: u64,
    pub(super) advanced_plugin_readbacks: SceneRendererAdvancedPluginReadbacks,
    pub(super) graph_resources: RenderGraphExecutionResources,
    pub(super) final_target_output: FinalTargetOutputSelection,
    pub(super) graph_execution_record: RenderGraphExecutionRecord,
    pub(super) graph_plugin_outputs: RenderPluginRendererOutputs,
}

impl SceneRendererCore {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare_compiled_scene_graph_frame<'a>(
        &mut self,
        backend: &'a RenderBackend,
        streamer: &mut ResourceStreamer,
        frame: &ViewportRenderFrame,
        target: &mut OffscreenTarget,
        pipeline: &CompiledRenderPipeline,
        runtime_features: SceneRuntimeFeatureFlags,
        history_textures: Option<&SceneFrameHistoryTextures>,
        history_availability: SceneHistoryAvailability,
        frame_generation: u64,
        generation_ids: RenderGenerationIds,
        gpu_pass_timer: &mut Option<&mut GpuPassTimer>,
        gpu_pipeline_statistics_timer: &mut Option<&mut GpuPipelineStatisticsTimer>,
        encoder: &mut wgpu::CommandEncoder,
        mesh_draw_lists: RenderPassMeshCommandLists<'_>,
        frame_buffer_uploads: &mut WgpuBufferUploadBatch,
        realtime_ibl_submission: &mut Option<RealtimeIblPendingSubmission>,
    ) -> Result<PreparedCompiledSceneGraphFrame<'a>, GraphicsError> {
        let device = &backend.device;
        let material_gbuffer_valid =
            pipeline_writes_resource(pipeline, PostProcessGraphResourceNames::GBUFFER_MATERIAL);
        let history_textures_present = history_textures.is_some();
        let taa_history_enabled = history_textures_present
            && frame.extract.view.anti_alias.mode == AntiAliasMode::Taa
            && pipeline_writes_resource(
                pipeline,
                PostProcessGraphResourceNames::TAA_HISTORY_CURRENT,
            );
        let screen_space_reflection_history_enabled = runtime_features.temporal_history_enabled
            && frame
                .extract
                .post_process
                .effect_stack
                .screen_space_reflection
                .is_enabled()
            && pipeline_writes_resource(
                pipeline,
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
            );
        let hzb_history_enabled =
            pipeline_writes_resource(pipeline, PostProcessGraphResourceNames::HZB_FURTHEST);
        let exposure_history_enabled =
            pipeline_writes_resource(pipeline, PostProcessGraphResourceNames::EXPOSURE_CURRENT);
        let exposure_history_reset_prepared = exposure_history_enabled
            && history_textures.is_some_and(|history| {
                history.prepare_exposure_history_reset(frame_buffer_uploads)
            });
        let volumetric_history_enabled = if pipeline_writes_resource(
            pipeline,
            PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING,
        ) {
            match crate::graphics::scene::scene_renderer::advanced_lighting::froxel::
                volumetric_history_quality(&frame.extract, frame.shader_quality())
            {
                Ok(quality) => quality.is_some(),
                Err(error) => {
                    abort_realtime_ibl_submission(&mut self.realtime_ibl, realtime_ibl_submission);
                    return Err(GraphicsError::Asset(error));
                }
            }
        } else {
            false
        };

        let product_diagnostic_frame_scope = if self
            .advanced_plugin_resources
            .has_runtime_prepare_gpu_readback_collectors()
        {
            backend
                .begin_product_diagnostic_readback_scope(frame_generation)
                .ok()
        } else {
            None
        };
        self.diagnostic_frame_index = self.diagnostic_frame_index.wrapping_add(1);
        let diagnostic_frame_index = self.diagnostic_frame_index;
        let product_diagnostic_query_scope = backend
            .begin_product_diagnostic_query_scope(
                generation_ids.timer_frame(),
                gpu_pass_timer.is_some(),
                gpu_pipeline_statistics_timer.is_some(),
            )
            .ok()
            .flatten();
        if let Some(scope) = product_diagnostic_query_scope.as_ref() {
            scope.attach_timers(
                gpu_pass_timer.as_deref_mut(),
                gpu_pipeline_statistics_timer.as_deref_mut(),
            );
        } else if let Some(timer) = gpu_pass_timer.as_deref_mut() {
            timer.defer_frame(generation_ids.timer_frame());
        }
        let mut advanced_plugin_readbacks = match self.execute_runtime_prepare_passes(
            backend.device_profile(),
            device,
            encoder,
            streamer,
            frame,
            product_diagnostic_frame_scope.is_some(),
            gpu_pass_timer.as_deref_mut(),
        ) {
            Ok(readbacks) => readbacks,
            Err(error) => {
                if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                    timer.defer_frame(generation_ids.timer_frame());
                }
                abort_realtime_ibl_submission(&mut self.realtime_ibl, realtime_ibl_submission);
                return Err(error);
            }
        };
        let mut graph_resources = RenderGraphExecutionResources::new();
        self.transient_resource_pool
            .begin_frame(backend.device_profile());
        self.transient_resource_pool
            .collect_completed_submissions(|ticket| backend.submission_status(ticket));
        let environment_source_cubemap_view = frame
            .environment()
            .skybox
            .source_cubemap_environment()
            .map(|_| self.scene_environment_cubemap.source_view());
        let final_target_output = match bind_compiled_scene_graph_resources(
            device,
            backend.device_profile(),
            pipeline,
            streamer,
            frame,
            target,
            &self.post_process,
            self.gpu_scene.light_buffer(),
            history_textures,
            CompiledSceneGraphResourceBindingFlags {
                taa_history_enabled,
                screen_space_reflection_history_enabled,
                hzb_history_enabled,
                exposure_history_enabled,
                volumetric_history_enabled,
                history_availability,
                runtime_features,
            },
            &mut graph_resources,
            &mut self.transient_resource_pool,
            &mut self.neutral_graph_buffers,
            mesh_draw_lists,
            self.hzb_occlusion_culler.as_ref(),
            &self.shadow_atlas_resources,
            advanced_plugin_readbacks.external_buffer_binding_packet(),
            environment_source_cubemap_view,
            frame_buffer_uploads,
        ) {
            Ok(output) => output,
            Err(error) => {
                if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                    timer.defer_frame(generation_ids.timer_frame());
                }
                abort_compiled_scene_graph_resource_frame(
                    &mut graph_resources,
                    &mut self.transient_resource_pool,
                );
                abort_realtime_ibl_submission(&mut self.realtime_ibl, realtime_ibl_submission);
                return Err(error);
            }
        };
        let materialization_report = match graph_resources
            .validate_materialized_graph_resources(pipeline.graph())
            .map_err(GraphicsError::Asset)
        {
            Ok(report) => report,
            Err(error) => {
                if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                    timer.defer_frame(generation_ids.timer_frame());
                }
                abort_compiled_scene_graph_resource_frame(
                    &mut graph_resources,
                    &mut self.transient_resource_pool,
                );
                abort_realtime_ibl_submission(&mut self.realtime_ibl, realtime_ibl_submission);
                return Err(error);
            }
        };
        let mut graph_execution_record = RenderGraphExecutionRecord::default();
        graph_execution_record.set_execution_batch_report(pipeline.execution_batch_report());
        graph_execution_record.set_materialization_report(materialization_report);
        graph_execution_record.set_resource_report(graph_resources.resource_report());
        graph_execution_record.set_resource_alias_report(graph_resources.resource_alias_report());
        for profile in advanced_plugin_readbacks.take_gpu_pass_profiles() {
            graph_execution_record.push_pass_profile_with_budget_key(
                profile.pass_name,
                profile.executor_id,
                profile.budget_key,
                profile.cpu_elapsed_micros,
            );
        }

        Ok(PreparedCompiledSceneGraphFrame {
            material_gbuffer_valid,
            taa_history_enabled,
            screen_space_reflection_history_enabled,
            hzb_history_enabled,
            exposure_history_enabled,
            exposure_history_reset_prepared,
            volumetric_history_enabled,
            product_diagnostic_frame_scope,
            product_diagnostic_query_scope,
            diagnostic_frame_index,
            advanced_plugin_readbacks,
            graph_resources,
            final_target_output,
            graph_execution_record,
            graph_plugin_outputs: RenderPluginRendererOutputs::default(),
        })
    }
}
