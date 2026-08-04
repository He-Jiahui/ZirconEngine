use crate::core::TaskPool;
use crate::core::framework::render::{
    FrameHistoryHandle, PostProcessGraphResourceNames, RenderCameraTargetGraphImportStatus,
    RenderCapabilitySummary, RenderCaptureReport, RenderCaptureSource, ShaderVariantMissReport,
};
#[cfg(test)]
use crate::core::{math::UVec2, resource::ResourceId};

use crate::graphics::CompiledRenderPipeline;
#[cfg(test)]
use crate::graphics::backend::read_texture_rgba;
use crate::graphics::backend::{
    DEFAULT_GPU_PIPELINE_STATISTICS_MAX_SCOPES, DEFAULT_GPU_TIMER_MAX_PASSES, GpuPassTimer,
    GpuPipelineStatisticsTimer, ViewportSurface,
};
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphLightGridReport;
use crate::graphics::scene::scene_renderer::mesh::PreparedMeshQueueStats;
use crate::graphics::scene::scene_renderer::sprite::PreparedSpriteQueueStats;
#[cfg(test)]
use crate::graphics::shader::ShaderVariantCacheDisk;
use crate::graphics::types::{GraphicsError, ViewportFrame, ViewportRenderFrame};
use crate::graphics::visibility::HzbOcclusionCullReport;
use crate::render_graph::{QueueLane, RenderGraphResourceAccessKind};

use super::super::runtime_features::runtime_features_from_pipeline;
use super::super::scene_renderer::{SceneRenderer, SceneRendererCaptureTarget};
use super::super::scene_renderer_history::prepare_history_textures;
use super::super::scene_renderer_runtime_outputs::{
    reset_last_runtime_outputs, store_last_runtime_outputs,
};
use super::super::scene_renderer_target::{ensure_offscreen_target, finish_viewport_frame};
use super::super::target_extent::viewport_size;
use super::{
    AsyncViewportCaptureRequest, ViewportAsyncCaptureSubmission, capture_request_was_admitted,
};

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn replace_shader_variant_disk_cache_for_tests(
        &mut self,
        cache: ShaderVariantCacheDisk,
    ) {
        self.core
            .mesh_pipelines
            .replace_shader_variant_disk_cache_for_tests(cache);
    }

    #[cfg(test)]
    pub(crate) fn read_output_target_texture_rgba_for_tests(
        &self,
        texture_id: &ResourceId,
    ) -> Result<Option<(UVec2, Vec<u8>)>, GraphicsError> {
        let Some(output_target) = self.streamer.output_target_texture_resource(texture_id) else {
            return Ok(None);
        };
        let size = output_target.size();
        let rgba = read_texture_rgba(
            &self.backend.device,
            &self.backend.queue,
            output_target.texture(),
            size,
        )?;
        Ok(Some((size, rgba)))
    }

    pub(crate) fn render_frame_with_pipeline(
        &mut self,
        frame: &ViewportRenderFrame,
        pipeline: &CompiledRenderPipeline,
        capabilities: &RenderCapabilitySummary,
        history_handle: Option<FrameHistoryHandle>,
        previous_history_available: bool,
    ) -> Result<ViewportFrame, GraphicsError> {
        self.render_frame_with_pipeline_optional_task_pool(
            frame,
            pipeline,
            capabilities,
            history_handle,
            previous_history_available,
            None,
            None,
        )
    }

    pub(crate) fn render_frame_with_pipeline_task_pool(
        &mut self,
        frame: &ViewportRenderFrame,
        pipeline: &CompiledRenderPipeline,
        capabilities: &RenderCapabilitySummary,
        history_handle: Option<FrameHistoryHandle>,
        previous_history_available: bool,
        task_pool: &TaskPool,
    ) -> Result<ViewportFrame, GraphicsError> {
        self.render_frame_with_pipeline_optional_task_pool(
            frame,
            pipeline,
            capabilities,
            history_handle,
            previous_history_available,
            Some(task_pool),
            None,
        )
    }

    pub(crate) fn render_frame_with_pipeline_async_capture_task_pool(
        &mut self,
        frame: &ViewportRenderFrame,
        pipeline: &CompiledRenderPipeline,
        capabilities: &RenderCapabilitySummary,
        history_handle: Option<FrameHistoryHandle>,
        previous_history_available: bool,
        task_pool: &TaskPool,
        capture: Option<AsyncViewportCaptureRequest>,
    ) -> Result<ViewportAsyncCaptureSubmission, GraphicsError> {
        let (generation, capture_admitted) = self.render_frame_with_pipeline_to_target(
            frame,
            pipeline,
            capabilities,
            history_handle,
            previous_history_available,
            Some(task_pool),
            capture,
        )?;
        let target = self.target.as_ref().expect("offscreen target");
        Ok(ViewportAsyncCaptureSubmission::new(
            generation,
            target.size,
            RenderCaptureReport::framework_offscreen(frame.output_target().kind(), target.size),
            capture_admitted,
        ))
    }

    fn render_frame_with_pipeline_optional_task_pool(
        &mut self,
        frame: &ViewportRenderFrame,
        pipeline: &CompiledRenderPipeline,
        capabilities: &RenderCapabilitySummary,
        history_handle: Option<FrameHistoryHandle>,
        previous_history_available: bool,
        task_pool: Option<&TaskPool>,
        viewport_capture: Option<AsyncViewportCaptureRequest>,
    ) -> Result<ViewportFrame, GraphicsError> {
        let (generation, _) = self.render_frame_with_pipeline_to_target(
            frame,
            pipeline,
            capabilities,
            history_handle,
            previous_history_available,
            task_pool,
            viewport_capture,
        )?;
        let target = self.target.as_ref().expect("offscreen target");
        if let Some((output_target, capture_report)) = output_target_capture_resource(
            &self.streamer,
            frame.output_target(),
            frame
                .camera_stack_output_policy()
                .owns_final_target_output(),
        ) {
            return finish_viewport_frame(
                &self.backend.device,
                &self.backend.queue,
                output_target.texture(),
                output_target.size(),
                generation,
                capture_report,
            );
        }
        finish_viewport_frame(
            &self.backend.device,
            &self.backend.queue,
            &target.final_color,
            target.size,
            generation,
            RenderCaptureReport::framework_offscreen(frame.output_target().kind(), target.size),
        )
    }

    pub(crate) fn present_frame_with_pipeline(
        &mut self,
        frame: &ViewportRenderFrame,
        pipeline: &CompiledRenderPipeline,
        capabilities: &RenderCapabilitySummary,
        history_handle: Option<FrameHistoryHandle>,
        previous_history_available: bool,
        surface: &mut ViewportSurface,
    ) -> Result<u64, GraphicsError> {
        self.present_frame_with_pipeline_optional_task_pool(
            frame,
            pipeline,
            capabilities,
            history_handle,
            previous_history_available,
            surface,
            None,
        )
    }

    pub(crate) fn present_frame_with_pipeline_task_pool(
        &mut self,
        frame: &ViewportRenderFrame,
        pipeline: &CompiledRenderPipeline,
        capabilities: &RenderCapabilitySummary,
        history_handle: Option<FrameHistoryHandle>,
        previous_history_available: bool,
        surface: &mut ViewportSurface,
        task_pool: &TaskPool,
    ) -> Result<u64, GraphicsError> {
        self.present_frame_with_pipeline_optional_task_pool(
            frame,
            pipeline,
            capabilities,
            history_handle,
            previous_history_available,
            surface,
            Some(task_pool),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn present_frame_with_pipeline_optional_task_pool(
        &mut self,
        frame: &ViewportRenderFrame,
        pipeline: &CompiledRenderPipeline,
        capabilities: &RenderCapabilitySummary,
        history_handle: Option<FrameHistoryHandle>,
        previous_history_available: bool,
        surface: &mut ViewportSurface,
        task_pool: Option<&TaskPool>,
    ) -> Result<u64, GraphicsError> {
        let (generation, _) = self.render_frame_with_pipeline_to_target(
            frame,
            pipeline,
            capabilities,
            history_handle,
            previous_history_available,
            task_pool,
            None,
        )?;
        let target = self.target.as_ref().expect("offscreen target");
        surface.present_texture(
            &self.backend.device,
            &self.backend.queue,
            &target.final_color_view,
        )?;
        Ok(generation)
    }

    fn render_frame_with_pipeline_to_target(
        &mut self,
        frame: &ViewportRenderFrame,
        pipeline: &CompiledRenderPipeline,
        capabilities: &RenderCapabilitySummary,
        history_handle: Option<FrameHistoryHandle>,
        previous_history_available: bool,
        task_pool: Option<&TaskPool>,
        viewport_capture: Option<AsyncViewportCaptureRequest>,
    ) -> Result<(u64, bool), GraphicsError> {
        reset_last_runtime_outputs(self);
        self.core.mesh_pipelines.reset_shader_variant_miss_report();
        self.core
            .mesh_pipelines
            .drain_pipeline_creation_diagnostics(&self.backend.device);
        self.last_gpu_timer_frame_result = {
            let (gpu_pass_timer, readback_queue) =
                (&mut self.gpu_pass_timer, &mut self.core.readback_queue);
            gpu_pass_timer
                .as_mut()
                .and_then(|timer| timer.try_collect(&self.backend.device, readback_queue))
        };
        self.last_gpu_pipeline_statistics_frame_result = {
            let (gpu_pipeline_statistics_timer, readback_queue) = (
                &mut self.gpu_pipeline_statistics_timer,
                &mut self.core.readback_queue,
            );
            gpu_pipeline_statistics_timer
                .as_mut()
                .and_then(|timer| timer.try_collect(&self.backend.device, readback_queue))
        };
        let frame_generation = self.generation.wrapping_add(1);

        self.streamer.ensure_scene_resources(
            &self.backend.device,
            &self.backend.queue,
            &self.core.texture_bind_group_layout,
            frame,
        )?;

        let size = viewport_size(frame);
        let render_size = frame.extract.view.effective_render_size();
        ensure_offscreen_target(&self.backend.device, &mut self.target, size, render_size);
        let runtime_features = runtime_features_from_pipeline(pipeline);
        let screen_space_reflection_history_enabled = runtime_features.temporal_history_enabled
            && frame
                .extract
                .post_process
                .effect_stack
                .screen_space_reflection
                .is_enabled()
            && pipeline_writes_screen_space_reflection_history(pipeline);
        let hzb_history_enabled =
            pipeline.writes_resource(PostProcessGraphResourceNames::HZB_FURTHEST);
        let exposure_history_enabled =
            pipeline.writes_resource(PostProcessGraphResourceNames::EXPOSURE_CURRENT);
        let volumetric_history_quality = pipeline
            .writes_resource(PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING)
        .then(|| {
            crate::graphics::scene::scene_renderer::advanced_lighting::froxel::volumetric_history_quality(
                &frame.extract,
                frame.shader_quality(),
            )
        })
        .transpose()
        .map_err(GraphicsError::Asset)?
        .flatten();

        let capture_admission = viewport_capture
            .as_ref()
            .map(AsyncViewportCaptureRequest::admission_state);
        let runtime_outputs = {
            let (history_textures, history_available) = prepare_history_textures(
                &self.backend.device,
                &self.backend.queue,
                &mut self.history_targets,
                history_handle,
                previous_history_available,
                size,
                render_size,
                runtime_features,
                screen_space_reflection_history_enabled,
                hzb_history_enabled,
                exposure_history_enabled,
                volumetric_history_quality,
            );
            let target = self.target.as_mut().expect("offscreen target");
            let parallel_record_min_passes_per_bucket = self.parallel_record_min_passes_per_bucket;
            let hzb_indirect_args_readback_enabled = self.hzb_indirect_args_readback_enabled;
            let (core, gpu_pass_timer, gpu_pipeline_statistics_timer) = (
                &mut self.core,
                self.gpu_pass_timer.as_mut(),
                self.gpu_pipeline_statistics_timer.as_mut(),
            );
            core.render_compiled_scene(
                &self.backend.device,
                &self.backend.queue,
                &self.streamer,
                frame,
                target,
                pipeline,
                capabilities,
                &self.render_pass_executors,
                runtime_features,
                history_textures,
                history_available,
                frame_generation,
                gpu_pass_timer,
                gpu_pipeline_statistics_timer,
                task_pool,
                parallel_record_min_passes_per_bucket,
                hzb_indirect_args_readback_enabled,
                viewport_capture,
            )?
        };
        let direct_imported = runtime_outputs
            .output_target_graph_import_report()
            .is_some_and(|report| {
                report.status == RenderCameraTargetGraphImportStatus::DirectImported
            });
        if !frame
            .camera_stack_output_policy()
            .owns_final_target_output()
        {
            self.streamer.suppress_output_target_writeback(frame);
        } else if direct_imported {
            self.streamer
                .skip_output_target_writeback_after_direct_import(frame);
        } else {
            let target = self.target.as_ref().expect("offscreen target");
            self.streamer.execute_output_target_writeback(
                &self.backend.device,
                &self.backend.queue,
                frame,
                &target.final_color,
                &target.final_color_view,
                target.size,
            )?;
        }
        store_last_runtime_outputs(self, runtime_outputs)?;
        self.last_capture_target = Some(SceneRendererCaptureTarget {
            output_target: frame.output_target(),
            owns_final_target_output: frame
                .camera_stack_output_policy()
                .owns_final_target_output(),
        });
        self.generation += 1;
        Ok((
            self.generation,
            capture_admission
                .as_deref()
                .is_some_and(capture_request_was_admitted),
        ))
    }
}

fn output_target_capture_resource(
    streamer: &crate::graphics::scene::resources::ResourceStreamer,
    output_target: crate::graphics::types::ViewportRenderOutputTarget,
    owns_final_target_output: bool,
) -> Option<(
    std::sync::Arc<crate::graphics::scene::resources::OutputTargetTextureResource>,
    RenderCaptureReport,
)> {
    if !owns_final_target_output {
        return None;
    }
    let graph_import_report = streamer.last_output_target_graph_import_report();
    let writeback_report = streamer.last_output_target_writeback_report();
    let texture = output_target.texture_handle()?;
    let output_target = streamer.output_target_texture_resource(&texture.id())?;
    let report = RenderCaptureReport::texture_from_reports(
        output_target.size(),
        graph_import_report,
        writeback_report,
    );
    matches!(
        report.source,
        RenderCaptureSource::TextureDirectGraphImport
            | RenderCaptureSource::TextureWritebackConversion
            | RenderCaptureSource::TextureWritebackCopy
    )
    .then_some((output_target, report))
}

fn pipeline_writes_screen_space_reflection_history(pipeline: &CompiledRenderPipeline) -> bool {
    pipeline.writes_resource(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY)
}

impl SceneRenderer {
    pub(crate) fn validate_compiled_pipeline_executors(
        &self,
        pipeline: &CompiledRenderPipeline,
    ) -> Result<(), String> {
        self.render_pass_executors
            .validate_compiled_pipeline(pipeline)
    }

    pub(crate) fn last_render_graph_executed_passes(&self) -> &[String] {
        self.last_render_graph_execution.executed_passes()
    }

    pub(crate) fn last_gpu_timer_frame_result(
        &self,
    ) -> Option<&crate::graphics::backend::GpuTimerFrameResult> {
        self.last_gpu_timer_frame_result.as_ref()
    }

    pub(crate) fn last_gpu_pipeline_statistics_frame_result(
        &self,
    ) -> Option<&crate::graphics::backend::GpuPipelineStatisticsFrameResult> {
        self.last_gpu_pipeline_statistics_frame_result.as_ref()
    }

    pub(crate) fn last_readback_poll_stats(&self) -> crate::graphics::backend::ReadbackPollStats {
        self.core.readback_queue.stats()
    }

    pub(crate) fn capture_latest_frame(&self) -> Result<Option<ViewportFrame>, GraphicsError> {
        let Some(target) = self.target.as_ref() else {
            return Ok(None);
        };
        let capture_target = self
            .last_capture_target
            .unwrap_or(SceneRendererCaptureTarget {
                output_target: Default::default(),
                owns_final_target_output: true,
            });
        if let Some((output_target, capture_report)) = output_target_capture_resource(
            &self.streamer,
            capture_target.output_target,
            capture_target.owns_final_target_output,
        ) {
            return finish_viewport_frame(
                &self.backend.device,
                &self.backend.queue,
                output_target.texture(),
                output_target.size(),
                self.generation,
                capture_report,
            )
            .map(Some);
        }
        finish_viewport_frame(
            &self.backend.device,
            &self.backend.queue,
            &target.final_color,
            target.size,
            self.generation,
            RenderCaptureReport::framework_offscreen(
                capture_target.output_target.kind(),
                target.size,
            ),
        )
        .map(Some)
    }

    pub(crate) fn poll_readback_completions(&mut self) {
        self.core
            .readback_queue
            .poll_completed(&self.backend.device);
    }

    pub(crate) fn wait_for_readback_completions(&mut self) -> Result<(), GraphicsError> {
        self.backend
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|error| GraphicsError::BufferMap(error.to_string()))?;
        self.poll_readback_completions();
        Ok(())
    }

    pub(crate) fn last_render_graph_executed_executor_ids(&self) -> &[String] {
        self.last_render_graph_execution.executed_executor_ids()
    }

    pub(crate) fn last_render_graph_executed_debug_markers(&self) -> &[String] {
        self.last_render_graph_execution.executed_debug_markers()
    }

    pub(crate) fn last_render_graph_executed_post_process_nodes(&self) -> &[String] {
        self.last_render_graph_execution
            .executed_post_process_nodes()
    }

    pub(crate) fn last_motion_vector_camera_status(
        &self,
    ) -> crate::core::framework::render::MotionVectorCameraStatus {
        self.last_render_graph_execution
            .motion_vector_camera_status()
    }

    pub(crate) fn last_render_graph_post_process_graph(
        &self,
    ) -> Option<&crate::core::framework::render::PostProcessPassGraph> {
        self.last_render_graph_execution.post_process_graph()
    }

    pub(crate) fn last_hzb_occlusion_cull_report(&self) -> Option<HzbOcclusionCullReport> {
        self.last_render_graph_execution.hzb_occlusion_cull_report()
    }

    pub(crate) fn last_light_grid_report(&self) -> Option<RenderGraphLightGridReport> {
        self.last_render_graph_execution.light_grid_report()
    }

    pub(crate) fn last_render_graph_executed_resource_access_count(&self) -> usize {
        self.last_render_graph_execution
            .executed_resource_access_count()
    }

    pub(crate) fn last_render_graph_executed_resource_access_count_for(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> usize {
        self.last_render_graph_execution
            .executed_resource_access_count_for(resource_name, access)
    }

    pub(crate) fn last_render_graph_executed_dependency_count(&self) -> usize {
        self.last_render_graph_execution.executed_dependency_count()
    }

    pub(crate) fn last_render_graph_compute_dispatch_count(&self) -> usize {
        self.last_render_graph_execution.compute_dispatch_count()
    }

    pub(crate) fn last_render_graph_compute_dispatch_group_count(&self) -> usize {
        self.last_render_graph_execution
            .compute_dispatch_group_volume_total()
    }

    pub(crate) fn last_render_graph_compute_dispatch_count_for_executor_prefix(
        &self,
        executor_prefix: &str,
    ) -> usize {
        self.last_render_graph_execution
            .compute_dispatch_count_for_executor_prefix(executor_prefix)
    }

    pub(crate) fn last_render_graph_compute_dispatch_group_count_for_executor_prefix(
        &self,
        executor_prefix: &str,
    ) -> usize {
        self.last_render_graph_execution
            .compute_dispatch_group_volume_total_for_executor_prefix(executor_prefix)
    }

    pub(crate) fn last_render_graph_compute_uploaded_bytes_for_executor_prefix(
        &self,
        executor_prefix: &str,
    ) -> u64 {
        self.last_render_graph_execution
            .compute_uploaded_bytes_total_for_executor_prefix(executor_prefix)
    }

    pub(crate) fn last_render_graph_compute_storage_write_resource_count(&self) -> usize {
        self.last_render_graph_execution
            .compute_storage_write_resource_count()
    }

    pub(crate) fn last_render_graph_compute_planned_workload_count(&self) -> usize {
        self.last_render_graph_execution
            .compute_workload_planned_count()
    }

    pub(crate) fn last_render_graph_compute_matched_workload_count(&self) -> usize {
        self.last_render_graph_execution
            .compute_workload_matched_count()
    }

    pub(crate) fn last_render_graph_compute_missing_dispatch_count(&self) -> usize {
        self.last_render_graph_execution
            .compute_workload_missing_dispatch_count()
    }

    pub(crate) fn last_render_graph_compute_workload_mismatch_count(&self) -> usize {
        self.last_render_graph_execution
            .compute_workload_mismatch_count()
    }

    pub(crate) fn last_render_graph_compute_unexpected_dispatch_count(&self) -> usize {
        self.last_render_graph_execution
            .compute_workload_unexpected_dispatch_count()
    }

    pub(crate) fn last_render_graph_execution_resource_report(
        &self,
    ) -> crate::core::framework::render::RenderGraphExecutionResourceReport {
        self.last_render_graph_execution.resource_report()
    }

    pub(crate) fn last_render_graph_materialization_report(
        &self,
    ) -> crate::core::framework::render::RenderGraphMaterializationReport {
        self.last_render_graph_execution.materialization_report()
    }

    pub(crate) fn last_render_graph_execution_alias_report(
        &self,
    ) -> &crate::core::framework::render::RenderGraphExecutionAliasReport {
        self.last_render_graph_execution.resource_alias_report()
    }

    pub(crate) fn last_render_graph_execution_profile_report(
        &self,
    ) -> crate::core::framework::render::RenderGraphExecutionProfileReport {
        self.last_render_graph_execution.profile_report()
    }

    pub(crate) fn last_render_graph_stage_execution_report(
        &self,
    ) -> crate::core::framework::render::RenderGraphStageExecutionReport {
        self.last_render_graph_execution.stage_execution_report()
    }

    pub(crate) fn last_frame_history_copy_report(
        &self,
    ) -> crate::core::framework::render::RenderHistoryCopyReport {
        self.last_render_graph_execution.history_copy_report()
    }

    pub(crate) fn last_scene_velocity_readback_report(
        &self,
    ) -> crate::core::framework::render::RenderSceneVelocityReadbackReport {
        self.last_render_graph_execution
            .scene_velocity_readback_report()
    }

    #[cfg(test)]
    pub(crate) fn last_scene_velocity_readback_rg16_float_bytes(&self) -> Option<Vec<u8>> {
        self.last_render_graph_execution
            .scene_velocity_readback_rg16_float_bytes()
            .map(<[u8]>::to_vec)
    }

    pub(crate) fn last_color_lut_readback_report(
        &self,
    ) -> crate::core::framework::render::RenderColorLutReadbackReport {
        self.last_render_graph_execution.color_lut_readback_report()
    }

    pub(crate) fn last_exposure_readback_report(
        &self,
    ) -> crate::core::framework::render::RenderExposureReadbackReport {
        self.last_render_graph_execution.exposure_readback_report()
    }

    pub(crate) fn last_render_graph_executed_queue_fallback_count(&self) -> usize {
        self.last_render_graph_execution
            .executed_queue_fallback_count()
    }

    pub(crate) fn last_render_graph_executed_queue_lane_count(&self, queue: QueueLane) -> usize {
        self.last_render_graph_execution
            .executed_queue_lane_count(queue)
    }

    pub(crate) fn last_render_graph_executed_stage_count(
        &self,
        stage: crate::graphics::pipeline::RenderPassStage,
    ) -> usize {
        self.last_render_graph_execution.executed_stage_count(stage)
    }

    pub(crate) fn last_prepared_mesh_queue_stats(&self) -> PreparedMeshQueueStats {
        self.last_prepared_mesh_queue_stats
    }

    pub(crate) fn last_shader_variant_miss_report(&self) -> ShaderVariantMissReport {
        self.core.mesh_pipelines.shader_variant_miss_report()
    }

    pub(crate) fn set_async_pipeline_compile_enabled(&mut self, enabled: bool) {
        self.core
            .mesh_pipelines
            .set_async_pipeline_compile_enabled(enabled);
    }

    pub(crate) fn async_pipeline_compile_enabled(&self) -> bool {
        self.core.mesh_pipelines.async_pipeline_compile_enabled()
    }

    pub(crate) fn set_gpu_pass_timing_enabled(&mut self, enabled: bool) {
        if enabled {
            if self.gpu_pass_timer.is_none() {
                self.gpu_pass_timer = GpuPassTimer::try_new(
                    &self.backend.device,
                    &self.backend.queue,
                    DEFAULT_GPU_TIMER_MAX_PASSES,
                );
                self.last_gpu_timer_frame_result = None;
            }
            if self.gpu_pipeline_statistics_timer.is_none() {
                self.gpu_pipeline_statistics_timer = GpuPipelineStatisticsTimer::try_new(
                    &self.backend.device,
                    DEFAULT_GPU_PIPELINE_STATISTICS_MAX_SCOPES,
                );
                self.last_gpu_pipeline_statistics_frame_result = None;
            }
        } else {
            self.gpu_pass_timer = None;
            self.gpu_pipeline_statistics_timer = None;
            self.last_gpu_timer_frame_result = None;
            self.last_gpu_pipeline_statistics_frame_result = None;
        }
    }

    pub(crate) const fn gpu_pass_timing_enabled(&self) -> bool {
        self.gpu_pass_timer.is_some()
    }

    pub(crate) fn last_pipeline_async_pending_count(&self) -> u32 {
        self.core
            .mesh_pipelines
            .async_pipeline_compile_pending_count()
    }

    pub(crate) fn last_variant_first_frame_miss_count(&self) -> u32 {
        self.core
            .mesh_pipelines
            .async_variant_first_frame_miss_count()
    }

    pub(crate) fn last_prepared_sprite_queue_stats(&self) -> PreparedSpriteQueueStats {
        self.last_prepared_sprite_queue_stats
    }

    pub(crate) fn last_ui_text_prepare_report(
        &self,
    ) -> crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiTextPrepareReport {
        self.core
            .screen_space_ui_renderer
            .as_ref()
            .map_or_else(Default::default, |renderer| renderer.text_prepare_report())
    }

    pub(crate) fn last_material_count(&self) -> usize {
        self.streamer.last_material_count()
    }

    pub(crate) fn last_material_ready_count(&self) -> usize {
        self.streamer.last_material_ready_count()
    }

    pub(crate) fn last_material_fallback_count(&self) -> usize {
        self.streamer.last_material_fallback_count()
    }

    pub(crate) fn last_material_validation_error_count(&self) -> usize {
        self.streamer.last_material_validation_error_count()
    }

    pub(crate) fn last_material_diagnostic_count(&self) -> usize {
        self.streamer.last_material_diagnostic_count()
    }

    pub(crate) fn last_sprite_count(&self) -> usize {
        self.streamer.last_sprite_count()
    }

    pub(crate) fn last_sprite_ready_count(&self) -> usize {
        self.streamer.last_sprite_ready_count()
    }

    pub(crate) fn last_sprite_texture_fallback_count(&self) -> usize {
        self.streamer.last_sprite_texture_fallback_count()
    }

    pub(crate) fn last_post_process_lut_request_count(&self) -> usize {
        self.streamer.last_post_process_lut_request_count()
    }

    pub(crate) fn last_post_process_lut_ready_count(&self) -> usize {
        self.streamer.last_post_process_lut_ready_count()
    }

    pub(crate) fn last_post_process_lut_fallback_count(&self) -> usize {
        self.streamer.last_post_process_lut_fallback_count()
    }

    pub(crate) fn last_post_process_lut_2d_strip_ready_count(&self) -> usize {
        self.streamer.last_post_process_lut_2d_strip_ready_count()
    }

    pub(crate) fn last_post_process_lut_3d_request_count(&self) -> usize {
        self.streamer.last_post_process_lut_3d_request_count()
    }

    pub(crate) fn last_post_process_lut_unsupported_shape_count(&self) -> usize {
        self.streamer
            .last_post_process_lut_unsupported_shape_count()
    }

    pub(crate) fn last_output_target_writeback_report(
        &self,
    ) -> crate::core::framework::render::RenderCameraTargetWritebackReport {
        self.streamer.last_output_target_writeback_report()
    }

    pub(crate) fn last_output_target_graph_import_report(
        &self,
    ) -> crate::core::framework::render::RenderCameraTargetGraphImportReport {
        self.streamer.last_output_target_graph_import_report()
    }
}
