use crate::core::TaskPool;
use crate::core::framework::render::{
    RenderCapabilitySummary, RenderFrameSubmissionProducer, RenderFrameSubmissionTransaction,
};
use crate::graphics::backend::{
    GpuPassTimer, GpuPipelineStatisticsTimer, OffscreenTarget, RenderBackend, ViewportSurface,
};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::core::scene_renderer_render_with_pipeline::AsyncViewportCaptureRequest;
use crate::graphics::scene::scene_renderer::graph_execution::{
    FrameCommandEncoderSet, RenderPassExecutorRegistry, TransientResourcePool,
};
use crate::graphics::scene::scene_renderer::history::{
    SceneFrameHistoryTextures, SceneHistoryFrameTransaction,
};
use crate::graphics::scene::scene_renderer::post_process::SceneRuntimeFeatureFlags;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::graphics::{CompiledRenderPipeline, EnvironmentIblBakeReservation};

use super::super::super::scene_renderer_core::SceneRendererCore;
use super::super::SceneRendererCompiledSceneOutputs;
use super::commit_compiled_scene_frame_success::CompiledSceneFrameSuccessContext;
use super::compiled_scene_frame_foundation::PreparedCompiledSceneFrameFoundation;
use super::execute_compiled_scene_graph_stages::CompiledSceneGraphStageContext;
use super::execute_graph_stage::RenderGraphStageExecution;
use super::frame_lifecycle::{
    abort_compiled_scene_graph_resource_frame, abort_realtime_ibl_submission,
};
use super::prepare_compiled_scene_graph_frame::PreparedCompiledSceneGraphFrame;
use super::prepare_compiled_scene_mesh_submission::{
    PreparedCompiledSceneMeshSubmission, project_compiled_scene_mesh_draw_lists,
};
use super::prepare_overlay_buffers::prepare_overlay_buffers;
use super::submit_compiled_scene_frame::{
    CompiledSceneFrameSubmissionContext, prepare_environment_ibl_runtime_cache_writeback,
};
use super::terminal_frame_packet::{TerminalFramePacketContext, prepare_terminal_frame_packet};

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
        mut history_textures: Option<&mut SceneFrameHistoryTextures>,
        mut history_frame_transaction: SceneHistoryFrameTransaction,
        history_initialization_command_buffer: Option<wgpu::CommandBuffer>,
        frame_generation: u64,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
        mut gpu_pass_timer: Option<&mut GpuPassTimer>,
        mut gpu_pipeline_statistics_timer: Option<&mut GpuPipelineStatisticsTimer>,
        compute_task_pool: Option<&TaskPool>,
        parallel_record_min_passes_per_bucket: Option<usize>,
        hzb_diagnostics_readback_enabled: bool,
        mut viewport_capture: Option<AsyncViewportCaptureRequest>,
        mut environment_ibl_bake_reservation: Option<EnvironmentIblBakeReservation>,
        viewport_product_copy: Option<&zr_rhi_wgpu::WgpuUiExternalImageCopyTarget>,
        surface_frame: Option<(&ViewportSurface, &zr_rhi_wgpu::WgpuNativeSurfaceFrameTarget)>,
    ) -> Result<SceneRendererCompiledSceneOutputs, GraphicsError> {
        self.ensure_device_epoch(backend)?;
        let device = &backend.device;
        let history_availability = history_frame_transaction.availability();
        let PreparedCompiledSceneFrameFoundation {
            mut encoder,
            mut frame_texture_uploads,
            mut frame_buffer_uploads,
            shadow_frame_plan,
            mut shadow_atlas_prepared_upload,
            mut realtime_ibl_submission,
            generation_ids,
            material_pipeline_features,
        } = self.prepare_compiled_scene_frame_foundation(
            backend,
            streamer,
            frame,
            target,
            pipeline,
            render_pass_executors,
            runtime_features,
            frame_generation,
        )?;
        let PreparedCompiledSceneMeshSubmission {
            compiled_scene_draws,
            gpu_scene_prepared_upload,
            mesh_pass_command_buffers,
            mesh_pass_indirect_draws,
            mesh_indirect_prepared_upload,
            prepared_mesh_queue_stats,
            prepared_sprite_queue_stats,
            mesh_draw_replay_stats,
        } = match self.prepare_compiled_scene_mesh_submission(
            backend,
            &mut encoder,
            streamer,
            frame,
            pipeline,
            capabilities,
            runtime_features,
            material_pipeline_features,
            &shadow_frame_plan,
            generation_ids,
            compute_task_pool,
            &mut frame_buffer_uploads,
        ) {
            Ok(prepared) => prepared,
            Err(error) => {
                abort_realtime_ibl_submission(&mut self.realtime_ibl, &mut realtime_ibl_submission);
                return Err(error);
            }
        };
        let gpu_scene_bind_group = self.gpu_scene.scene_bind_group().clone();
        let mesh_draw_lists = project_compiled_scene_mesh_draw_lists(
            &mesh_draw_replay_stats,
            &gpu_scene_bind_group,
            &mesh_pass_command_buffers,
            &mesh_pass_indirect_draws,
            frame
                .extract
                .lighting
                .advanced_lighting
                .transmission_draw_step_count(),
        );
        let prepared_overlays = match prepare_overlay_buffers(
            self,
            device,
            streamer,
            frame,
            &mut frame_texture_uploads,
        ) {
            Ok(prepared_overlays) => prepared_overlays,
            Err(error) => {
                abort_realtime_ibl_submission(&mut self.realtime_ibl, &mut realtime_ibl_submission);
                return Err(error);
            }
        };
        let PreparedCompiledSceneGraphFrame {
            material_gbuffer_valid,
            taa_history_enabled,
            screen_space_reflection_history_enabled,
            hzb_history_enabled,
            exposure_history_enabled,
            exposure_history_reset_prepared,
            volumetric_history_enabled,
            mut product_diagnostic_frame_scope,
            product_diagnostic_query_scope,
            diagnostic_frame_index,
            mut advanced_plugin_readbacks,
            mut graph_resources,
            final_target_output,
            mut graph_execution_record,
            mut graph_plugin_outputs,
        } = self.prepare_compiled_scene_graph_frame(
            backend,
            streamer,
            frame,
            target,
            pipeline,
            runtime_features,
            history_textures.as_deref(),
            history_availability,
            frame_generation,
            generation_ids,
            &mut gpu_pass_timer,
            &mut gpu_pipeline_statistics_timer,
            &mut encoder,
            mesh_draw_lists,
            &mut frame_buffer_uploads,
            &mut realtime_ibl_submission,
        )?;
        let mut runtime_prepare_buffer_uploads =
            advanced_plugin_readbacks.take_runtime_prepare_buffer_uploads();
        frame_buffer_uploads.append(&mut runtime_prepare_buffer_uploads);
        let mut graph_execution = RenderGraphStageExecution::new(
            &mut graph_resources,
            &mut graph_execution_record,
            &mut graph_plugin_outputs,
            gpu_pass_timer.as_deref_mut(),
            gpu_pipeline_statistics_timer.as_deref_mut(),
        )
        .with_output_target_writeback_plan(final_target_output.writeback_plan());
        let mut command_encoders = FrameCommandEncoderSet::from_serial_encoder(encoder);
        let parallel_recording = compute_task_pool.zip(parallel_record_min_passes_per_bucket);
        let graph_execution_result =
            self.execute_compiled_scene_graph_stages(CompiledSceneGraphStageContext {
                device,
                command_encoders: &mut command_encoders,
                streamer,
                frame,
                surface_frame,
                target,
                pipeline,
                render_pass_executors,
                runtime_features,
                graph_execution: &mut graph_execution,
                mesh_draw_lists,
                history_textures: history_textures.as_deref(),
                history_frame_transaction: &mut history_frame_transaction,
                history_availability,
                material_gbuffer_valid,
                taa_history_enabled,
                screen_space_reflection_history_enabled,
                hzb_history_enabled,
                exposure_history_enabled,
                volumetric_history_enabled,
                shadow_frame_plan: &shadow_frame_plan,
                prepared_overlays: &prepared_overlays,
                parallel_recording,
            });
        if let Err(error) = graph_execution_result {
            drop(graph_execution);
            if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                timer.defer_frame(generation_ids.timer_frame());
            }
            abort_realtime_ibl_submission(&mut self.realtime_ibl, &mut realtime_ibl_submission);
            drop(command_encoders);
            abort_compiled_scene_graph_resource_frame(
                &mut graph_resources,
                &mut self.transient_resource_pool,
            );
            return Err(error);
        }
        let mut graph_buffer_uploads = graph_execution.take_buffer_uploads();
        let graph_texture_uploads = graph_execution.take_texture_uploads();
        let screen_space_ui_upload_commits = graph_execution.take_screen_space_ui_upload_commits();
        let hzb_occlusion_params_commits = graph_execution.take_hzb_occlusion_params_commits();
        let output_target_writeback_report = graph_execution
            .take_output_target_writeback_report()
            .unwrap_or_else(|| {
                crate::core::framework::render::RenderCameraTargetWritebackReport::not_requested(
                    frame.output_target().kind(),
                )
            });
        drop(graph_execution);
        graph_execution_record.finalize_ambient_occlusion_report(frame_generation, pipeline);
        streamer.set_output_target_writeback_report(output_target_writeback_report);
        frame_buffer_uploads.append(&mut graph_buffer_uploads);
        frame_texture_uploads.append(graph_texture_uploads);
        let frame_resource_upload_submission = match backend.enqueue_copy_resource_upload_batch(
            zr_rhi_wgpu::WgpuResourceUploadBatch::from_batches(
                frame_buffer_uploads,
                frame_texture_uploads,
            ),
        ) {
            Ok(submission) => submission,
            Err(error) => {
                if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                    timer.defer_frame(generation_ids.timer_frame());
                }
                abort_realtime_ibl_submission(&mut self.realtime_ibl, &mut realtime_ibl_submission);
                drop(command_encoders);
                abort_compiled_scene_graph_resource_frame(
                    &mut graph_resources,
                    &mut self.transient_resource_pool,
                );
                return Err(error);
            }
        };
        if let Err(error) = backend.record_pre_scene_submission(
            submission_transaction,
            RenderFrameSubmissionProducer::FrameResourceUpload,
            frame_resource_upload_submission,
        ) {
            if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                timer.defer_frame(generation_ids.timer_frame());
            }
            abort_realtime_ibl_submission(&mut self.realtime_ibl, &mut realtime_ibl_submission);
            drop(command_encoders);
            abort_compiled_scene_graph_resource_frame(
                &mut graph_resources,
                &mut self.transient_resource_pool,
            );
            return Err(error);
        }
        let hzb_readback_requested = hzb_diagnostics_readback_enabled
            && graph_execution_record
                .hzb_occlusion_cull_report()
                .is_some_and(|report| report.dispatched_phase_count > 0);
        let environment_ibl_artifact_readback_requested = environment_ibl_bake_reservation
            .is_some()
            && pipeline.environment_ibl_bake_request.is_some();
        let mut environment_ibl_prepare_error = None;
        let product_diagnostic_frame_requested = product_diagnostic_frame_scope.is_some()
            || viewport_capture.is_some()
            || hzb_readback_requested
            || environment_ibl_artifact_readback_requested
            || (realtime_ibl_submission.is_some() && self.realtime_ibl.gpu_timestamps_supported());
        if product_diagnostic_frame_scope.is_none() && product_diagnostic_frame_requested {
            match backend.begin_product_diagnostic_readback_scope(frame_generation) {
                Ok(scope) => product_diagnostic_frame_scope = Some(scope),
                Err(error) if viewport_capture.is_some() => {
                    abort_realtime_ibl_submission(
                        &mut self.realtime_ibl,
                        &mut realtime_ibl_submission,
                    );
                    drop(command_encoders);
                    abort_compiled_scene_graph_resource_frame(
                        &mut graph_resources,
                        &mut self.transient_resource_pool,
                    );
                    return Err(error);
                }
                Err(error) => {
                    if hzb_readback_requested {
                        if let Some(culler) = self.hzb_occlusion_culler.as_ref() {
                            culler.record_skipped_readback();
                        }
                    }
                    if environment_ibl_artifact_readback_requested {
                        environment_ibl_prepare_error = Some(error);
                        environment_ibl_bake_reservation.take();
                    }
                }
            }
        }
        let mut prepared_ibl_writeback = None;
        if product_diagnostic_frame_scope.is_some() {
            if let Some(viewport_capture) = viewport_capture.take() {
                let (callback, admission) = viewport_capture.into_parts();
                let admitted = match backend.enqueue_product_diagnostic_texture_rgba8(
                    &target.final_color,
                    target.size.x,
                    target.size.y,
                    callback,
                ) {
                    Ok(admitted) => admitted,
                    Err(error) => {
                        drop(product_diagnostic_frame_scope.take());
                        abort_realtime_ibl_submission(
                            &mut self.realtime_ibl,
                            &mut realtime_ibl_submission,
                        );
                        drop(command_encoders);
                        abort_compiled_scene_graph_resource_frame(
                            &mut graph_resources,
                            &mut self.transient_resource_pool,
                        );
                        return Err(error);
                    }
                };
                if admitted {
                    admission.store(true, std::sync::atomic::Ordering::Release);
                }
            }
            if let Err(error) = advanced_plugin_readbacks.register_product_gpu_readbacks(backend) {
                drop(product_diagnostic_frame_scope.take());
                abort_realtime_ibl_submission(&mut self.realtime_ibl, &mut realtime_ibl_submission);
                drop(command_encoders);
                abort_compiled_scene_graph_resource_frame(
                    &mut graph_resources,
                    &mut self.transient_resource_pool,
                );
                return Err(error);
            }
            if hzb_readback_requested {
                if let Some(culler) = self.hzb_occlusion_culler.as_ref() {
                    if let Err(error) = culler.request_frame_readbacks(
                        backend,
                        &mesh_pass_indirect_draws,
                        diagnostic_frame_index,
                    ) {
                        drop(product_diagnostic_frame_scope.take());
                        abort_realtime_ibl_submission(
                            &mut self.realtime_ibl,
                            &mut realtime_ibl_submission,
                        );
                        drop(command_encoders);
                        abort_compiled_scene_graph_resource_frame(
                            &mut graph_resources,
                            &mut self.transient_resource_pool,
                        );
                        return Err(error);
                    }
                }
            }
            if let Some(submission) = realtime_ibl_submission.as_ref() {
                self.realtime_ibl.request_product_gpu_timestamp_readback(
                    submission,
                    backend.render_device.timestamp_period_ns(),
                    backend,
                );
            }
            if environment_ibl_prepare_error.is_none() {
                match prepare_environment_ibl_runtime_cache_writeback(
                    &self.ibl_bake_runtime_writebacks,
                    backend,
                    streamer,
                    pipeline.environment_ibl_bake_request,
                    environment_ibl_bake_reservation.take(),
                    &graph_resources,
                    pipeline.graph(),
                ) {
                    Ok(prepared) => prepared_ibl_writeback = prepared,
                    Err(error) => environment_ibl_prepare_error = Some(error),
                }
            }
        } else if hzb_readback_requested {
            if let Some(culler) = self.hzb_occlusion_culler.as_ref() {
                culler.record_skipped_readback();
            }
        }
        let terminal_frame_packet = prepare_terminal_frame_packet(TerminalFramePacketContext {
            device,
            target,
            command_encoders,
            history_initialization_command_buffer,
            viewport_product_copy,
            product_diagnostic_frame_scope,
            product_diagnostic_query_scope,
            gpu_pass_timer,
            gpu_pipeline_statistics_timer,
            timer_frame_generation: generation_ids.timer_frame(),
        });
        let terminal_frame_packet = match terminal_frame_packet {
            Ok(packet) => packet,
            Err(error) => {
                abort_realtime_ibl_submission(&mut self.realtime_ibl, &mut realtime_ibl_submission);
                abort_compiled_scene_graph_resource_frame(
                    &mut graph_resources,
                    &mut self.transient_resource_pool,
                );
                return Err(error);
            }
        };

        let scene_submission =
            self.submit_compiled_scene_frame(CompiledSceneFrameSubmissionContext {
                backend,
                #[cfg(test)]
                device,
                #[cfg(test)]
                queue: &backend.queue,
                command_buffers: terminal_frame_packet.command_buffers,
                #[cfg(test)]
                streamer,
                #[cfg(test)]
                frame,
                graph_resources: &mut graph_resources,
                graph_execution_record: &mut graph_execution_record,
                prepared_ibl_writeback,
                environment_ibl_prepare_error,
                realtime_ibl_submission,
                diagnostic_frame_index,
                product_diagnostic_frame: terminal_frame_packet.product_diagnostic_frame,
                product_diagnostic_query_frame: terminal_frame_packet
                    .product_diagnostic_query_frame,
                surface_target: surface_frame.map(|(_, target)| target),
                history_textures,
                history_frame_transaction,
                frame_generation,
                exposure_history_reset_prepared,
            })?;
        if let Err(source) = submission_transaction.validate_scene_submission(scene_submission) {
            return Err(GraphicsError::FrameFailedAfterSceneSubmission {
                scene_submission,
                source: Box::new(source.into()),
            });
        }
        Ok(
            self.commit_compiled_scene_frame_success(CompiledSceneFrameSuccessContext {
                hzb_occlusion_params_commits,
                screen_space_ui_upload_commits,
                mesh_indirect_prepared_upload,
                shadow_atlas_prepared_upload,
                gpu_scene_prepared_upload,
                advanced_plugin_readbacks,
                graph_plugin_outputs,
                graph_execution_record,
                prepared_mesh_queue_stats,
                prepared_sprite_queue_stats,
                mesh_draw_replay_stats,
                compiled_scene_draws,
                final_target_output,
                scene_submission,
            }),
        )
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
