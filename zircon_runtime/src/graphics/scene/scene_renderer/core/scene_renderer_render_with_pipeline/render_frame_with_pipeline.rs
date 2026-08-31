use crate::core::TaskPool;
use crate::core::framework::render::{
    FrameHistoryHandle, RenderCapabilitySummary, RenderCaptureReport, RenderCaptureSource,
    RenderFrameHistoryInput, RenderGpuTimingStatus, RenderReflectionProbeWorkloadReport,
    ShaderVariantMissReport,
};
#[cfg(test)]
use crate::core::{math::UVec2, resource::ResourceId};

#[cfg(test)]
use crate::graphics::backend::read_texture_rgba;
use crate::graphics::backend::{
    DEFAULT_GPU_PIPELINE_STATISTICS_MAX_SCOPES, DEFAULT_GPU_TIMER_MAX_PASSES, GpuPassTimer,
    GpuPipelineStatisticsTimer,
};
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphLightGridReport;
use crate::graphics::scene::scene_renderer::mesh::PreparedMeshQueueStats;
use crate::graphics::scene::scene_renderer::sprite::PreparedSpriteQueueStats;
#[cfg(test)]
use crate::graphics::shader::ShaderVariantCacheDisk;
use crate::graphics::types::{GraphicsError, ViewportFrame, ViewportRenderFrame};
use crate::graphics::visibility::HzbOcclusionCullReport;
use crate::graphics::{CompiledRenderPipeline, EnvironmentIblBakeReservation};
use crate::render_graph::{QueueLane, RenderGraphResourceAccessKind};

use super::super::scene_renderer::SceneRenderer;
use super::super::scene_renderer_completion::route_frame_submission_completion_consumers;
use super::super::scene_renderer_target::{finish_viewport_frame, require_offscreen_target};
use super::{AsyncViewportCaptureRequest, ViewportAsyncCaptureSubmission};

mod frame_submission_owner;
mod gpu_timing_status;
mod readback;
mod surface_presentation;
#[cfg(test)]
#[path = "render_frame_with_pipeline/tests.rs"]
mod tests;

pub(in crate::graphics::scene::scene_renderer::core) use gpu_timing_status::render_gpu_timing_status;

impl SceneRenderer {
    pub(crate) fn reflection_probe_workload_report(&self) -> RenderReflectionProbeWorkloadReport {
        self.core
            .mesh_pipelines
            .reflection_probes
            .last_workload_report()
    }

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

    #[cfg(test)]
    pub(crate) fn reflection_probe_upload_diagnostics_for_tests(
        &self,
    ) -> (usize, usize, usize, usize, Option<String>) {
        self.core
            .mesh_pipelines
            .reflection_probes
            .last_report_diagnostics()
    }

    #[cfg(test)]
    pub(crate) fn reflection_probe_gpu_upload_diagnostics_for_tests(
        &self,
    ) -> Result<(u32, [[f32; 4]; 2], [[u16; 4]; 2]), GraphicsError> {
        self.core
            .mesh_pipelines
            .reflection_probes
            .gpu_upload_diagnostics(&self.backend.device, &self.backend.queue)
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
            RenderFrameHistoryInput::new(history_handle, previous_history_available, None),
            None,
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
            RenderFrameHistoryInput::new(history_handle, previous_history_available, None),
            Some(task_pool),
            None,
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
        self.render_frame_with_pipeline_async_capture_task_pool_with_environment_ibl_bake_reservation(
            frame,
            pipeline,
            capabilities,
            RenderFrameHistoryInput::new(history_handle, previous_history_available, None),
            task_pool,
            capture,
            None,
            false,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_frame_with_pipeline_async_capture_task_pool_with_environment_ibl_bake_reservation(
        &mut self,
        frame: &ViewportRenderFrame,
        pipeline: &CompiledRenderPipeline,
        capabilities: &RenderCapabilitySummary,
        history_input: RenderFrameHistoryInput,
        task_pool: &TaskPool,
        capture: Option<AsyncViewportCaptureRequest>,
        environment_ibl_bake_reservation: Option<EnvironmentIblBakeReservation>,
        viewport_product_requested: bool,
    ) -> Result<ViewportAsyncCaptureSubmission, GraphicsError> {
        let (submission_receipt, capture_admitted, viewport_product_copy) = self
            .render_frame_with_pipeline_to_target(
                frame,
                pipeline,
                capabilities,
                history_input,
                Some(task_pool),
                capture,
                environment_ibl_bake_reservation,
                viewport_product_requested,
                None,
                None,
            )?;
        let target = require_offscreen_target(self.target.as_ref())?;
        Ok(ViewportAsyncCaptureSubmission::new(
            submission_receipt,
            viewport_product_copy,
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
        history_input: RenderFrameHistoryInput,
        task_pool: Option<&TaskPool>,
        viewport_capture: Option<AsyncViewportCaptureRequest>,
        environment_ibl_bake_reservation: Option<EnvironmentIblBakeReservation>,
    ) -> Result<ViewportFrame, GraphicsError> {
        let (submission_receipt, _, _) = self.render_frame_with_pipeline_to_target(
            frame,
            pipeline,
            capabilities,
            history_input,
            task_pool,
            viewport_capture,
            environment_ibl_bake_reservation,
            false,
            None,
            None,
        )?;
        let target = require_offscreen_target(self.target.as_ref())?;
        let output_target_capture = output_target_capture_resource(
            &self.streamer,
            frame.output_target(),
            frame
                .camera_stack_output_policy()
                .owns_final_target_output(),
        );
        let backend = &self.backend;
        let mut observe_poll = |poll_receipt| {
            route_frame_submission_completion_consumers(
                backend,
                &mut self.core,
                &mut self.scene_submission_completion_journal,
                &mut self.gpu_pass_timer,
                &mut self.gpu_pipeline_statistics_timer,
                &mut self.last_gpu_timer_frame_result,
                &mut self.last_gpu_pipeline_statistics_frame_result,
                poll_receipt,
            )
        };
        if let Some((output_target, capture_report)) = output_target_capture {
            return finish_viewport_frame(
                backend,
                output_target.texture(),
                output_target.size(),
                submission_receipt.frame_generation(),
                capture_report,
                &mut observe_poll,
            );
        }
        finish_viewport_frame(
            backend,
            &target.final_color,
            target.size,
            submission_receipt.frame_generation(),
            RenderCaptureReport::framework_offscreen(frame.output_target().kind(), target.size),
            &mut observe_poll,
        )
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

    /// Drains one completed timestamp frame without exposing the backend query type.
    pub fn take_completed_gpu_timing_report(
        &mut self,
    ) -> Option<super::super::scene_renderer::SceneRendererGpuTimingReport> {
        let timestamp_period_ns = self
            .gpu_pass_timer
            .as_ref()
            .map(GpuPassTimer::timestamp_period_ns)?;
        self.last_gpu_timer_frame_result.take().map(|frame| {
            super::super::scene_renderer::SceneRendererGpuTimingReport::new(
                frame.frame_generation,
                timestamp_period_ns,
                frame.pass_timings.into_iter().map(|timing| {
                    super::super::scene_renderer::SceneRendererGpuPassTiming::new(
                        timing.pass_name,
                        timing.gpu_time_us,
                    )
                }),
            )
        })
    }

    pub const fn last_gpu_timing_status(&self) -> RenderGpuTimingStatus {
        self.last_gpu_timing_status
    }

    pub(crate) fn last_gpu_pipeline_statistics_frame_result(
        &self,
    ) -> Option<&crate::graphics::backend::GpuPipelineStatisticsFrameResult> {
        self.last_gpu_pipeline_statistics_frame_result.as_ref()
    }

    pub(crate) fn ui_surface_context(&self) -> zr_rhi_wgpu::WgpuUiSurfaceContext {
        self.backend.ui_surface_context()
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

    pub(crate) fn last_taa_reactive_mask_encoding(&self) -> (usize, u64) {
        self.last_render_graph_execution
            .taa_reactive_mask_encoding()
    }

    pub(crate) fn last_taa_resolve_bind_group_create_count(&self) -> usize {
        self.last_render_graph_execution
            .taa_resolve_bind_group_create_count()
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

    pub(crate) fn last_render_graph_parallel_recording_report(
        &self,
    ) -> crate::core::framework::render::RenderGraphParallelRecordingReport {
        self.last_render_graph_execution.parallel_recording_report()
    }

    pub(crate) fn last_render_graph_execution_batch_report(
        &self,
    ) -> crate::core::framework::render::RenderGraphExecutionBatchReport {
        self.last_render_graph_execution.execution_batch_report()
    }

    pub(crate) fn last_render_graph_stage_execution_report(
        &self,
    ) -> crate::core::framework::render::RenderGraphStageExecutionReport {
        self.last_render_graph_execution.stage_execution_report()
    }

    pub(crate) fn last_ambient_occlusion_execution_report(
        &self,
    ) -> crate::core::framework::render::RenderAmbientOcclusionExecutionReport {
        self.last_render_graph_execution
            .ambient_occlusion_execution_report()
    }

    pub(crate) fn last_scene_submission_completion_report(
        &self,
    ) -> crate::core::framework::render::RenderSceneSubmissionCompletionReport {
        self.scene_submission_completion_journal.last_report()
    }

    pub(crate) fn last_frame_history_copy_report(
        &self,
    ) -> crate::core::framework::render::RenderHistoryCopyReport {
        self.last_render_graph_execution.history_copy_report()
    }

    pub(crate) fn last_frame_history_domains_report(
        &self,
    ) -> crate::core::framework::render::RenderHistoryDomainsReport {
        self.last_render_graph_execution.history_domains_report()
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

    /// Returns current-frame variant counters plus current residency and renderer-lifetime costs.
    pub fn last_shader_variant_miss_report(&self) -> ShaderVariantMissReport {
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
        self.gpu_pass_timing_requested = enabled;
        if enabled {
            if self.gpu_pass_timer.is_none() {
                self.gpu_pass_timer = GpuPassTimer::try_new_product(
                    &self.backend.device,
                    self.backend.render_device.timestamp_period_ns(),
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
            self.last_gpu_timing_status = if self.gpu_pass_timer.is_some() {
                RenderGpuTimingStatus::Pending
            } else {
                RenderGpuTimingStatus::Unavailable
            };
        } else {
            self.gpu_pass_timer = None;
            self.gpu_pipeline_statistics_timer = None;
            self.last_gpu_timer_frame_result = None;
            self.last_gpu_pipeline_statistics_frame_result = None;
            self.last_gpu_timing_status = RenderGpuTimingStatus::Disabled;
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
