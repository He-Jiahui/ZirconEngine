use crate::core::TaskPool;
use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderCapabilitySummary, RenderFrameHistoryInput,
    RenderFrameSubmissionReceipt, RenderFrameSubmissionTransaction, RenderPipelinePhase,
};
use crate::graphics::backend::{GpuPassTimer, ViewportSurface};
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::graphics::{CompiledRenderPipeline, EnvironmentIblBakeReservation};
use crate::rhi::SubmissionPollReceipt;
use zr_rhi_wgpu::WgpuNativeSurfaceFrameTarget;

use super::super::super::runtime_features::runtime_features_from_pipeline;
use super::super::super::scene_renderer::{SceneRenderer, SceneRendererCaptureTarget};
use super::super::super::scene_renderer_history::prepare_history_textures;
use super::super::super::scene_renderer_runtime_outputs::{
    reset_last_runtime_outputs, store_last_runtime_outputs,
};
use super::super::super::scene_renderer_submission_failure::settle_failed_frame_submissions;
use super::super::super::scene_renderer_target::{
    ensure_offscreen_target, require_offscreen_target_mut,
};
use super::super::super::target_extent::viewport_size;
use super::super::{AsyncViewportCaptureRequest, capture_request_was_admitted};
use super::render_gpu_timing_status;

impl SceneRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_frame_with_pipeline_to_target(
        &mut self,
        frame: &ViewportRenderFrame,
        pipeline: &CompiledRenderPipeline,
        capabilities: &RenderCapabilitySummary,
        history_input: RenderFrameHistoryInput,
        task_pool: Option<&TaskPool>,
        viewport_capture: Option<AsyncViewportCaptureRequest>,
        environment_ibl_bake_reservation: Option<EnvironmentIblBakeReservation>,
        viewport_product_requested: bool,
        surface_frame: Option<(&ViewportSurface, &WgpuNativeSurfaceFrameTarget)>,
        poll_receipt: Option<SubmissionPollReceipt>,
    ) -> Result<
        (
            RenderFrameSubmissionReceipt,
            bool,
            Option<zr_rhi_wgpu::WgpuUiExternalImageCopyReceipt>,
        ),
        GraphicsError,
    > {
        reset_last_runtime_outputs(self);
        let frame_generation = self.generation.wrapping_add(1);
        let poll_receipt = match poll_receipt {
            Some(poll_receipt) => poll_receipt,
            None => self.poll_frame_submission_completions()?,
        };
        let submission_metrics_baseline = self.backend.submission_metrics();
        let mut submission_transaction =
            RenderFrameSubmissionTransaction::begin(frame_generation, poll_receipt);
        self.core.mesh_pipelines.reset_shader_variant_miss_report();
        self.core
            .mesh_pipelines
            .drain_pipeline_creation_diagnostics();
        if let Err(source) = self.streamer.ensure_scene_resources(
            &self.backend,
            &self.backend.device,
            &self.core.texture_bind_group_layout,
            frame,
            &mut submission_transaction,
        ) {
            return Err(settle_failed_frame_submissions(
                &self.backend,
                &mut self.streamer,
                submission_transaction,
                source,
            ));
        }

        let size = viewport_size(frame);
        let render_size = match frame
            .view_family_pipeline()
            .output_target_for_phase(RenderPipelinePhase::SceneLinear)
        {
            Some(target) => target.allocation_extent(),
            None => {
                return Err(settle_failed_frame_submissions(
                    &self.backend,
                    &mut self.streamer,
                    submission_transaction,
                    GraphicsError::MissingViewFamilyPhase {
                        phase: RenderPipelinePhase::SceneLinear,
                    },
                ));
            }
        };
        let history_size = frame
            .view_family_pipeline()
            .temporal_history_key()
            .map(|key| key.history_allocation_extent())
            .or_else(|| {
                frame
                    .view_family_pipeline()
                    .phase_targets(RenderPipelinePhase::PostReconstructionScenePostProcess)
                    .map(|targets| targets.output().allocation_extent())
            })
            .unwrap_or(size);
        if ensure_offscreen_target(&self.backend.device, &mut self.target, size, render_size) {
            self.core
                .post_process
                .invalidate_taa_resolve_bind_group_cache();
        }
        let runtime_features = runtime_features_from_pipeline(pipeline);
        let taa_history_enabled = runtime_features.temporal_history_enabled
            && frame.extract.view.anti_alias.mode
                == crate::core::framework::render::AntiAliasMode::Taa
            && pipeline.writes_resource(PostProcessGraphResourceNames::TAA_HISTORY_CURRENT);
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
        let volumetric_history_quality_result = pipeline
            .writes_resource(PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING)
            .then(|| {
                crate::graphics::scene::scene_renderer::advanced_lighting::froxel::volumetric_history_quality(
                    &frame.extract,
                    frame.shader_quality(),
                )
            })
            .transpose()
            .map_err(GraphicsError::Asset);
        let volumetric_history_quality = match volumetric_history_quality_result {
            Ok(quality) => quality.flatten(),
            Err(source) => {
                return Err(settle_failed_frame_submissions(
                    &self.backend,
                    &mut self.streamer,
                    submission_transaction,
                    source,
                ));
            }
        };

        let capture_admission = viewport_capture
            .as_ref()
            .map(AsyncViewportCaptureRequest::admission_state);
        let viewport_product_copy = viewport_product_requested
            .then(|| {
                self.backend
                    .ui_surface_context()
                    .prepare_texture_for_external_image(
                        size.x,
                        size.y,
                        crate::graphics::scene::scene_renderer::FINAL_COLOR_FORMAT,
                        frame_generation,
                    )
            })
            .transpose()
            .map_err(GraphicsError::from);
        let viewport_product_copy = match viewport_product_copy {
            Ok(copy) => copy,
            Err(source) => {
                return Err(settle_failed_frame_submissions(
                    &self.backend,
                    &mut self.streamer,
                    submission_transaction,
                    source,
                ));
            }
        };
        let mut history_initialization_needs_abort_cleanup = false;
        let runtime_outputs_result = (|| -> Result<_, GraphicsError> {
            let (
                history_textures,
                history_frame_transaction,
                taa_history_allocation_changed,
                history_initialization_command_buffer,
            ) = prepare_history_textures(
                &self.backend,
                &mut self.history_targets,
                history_input,
                history_size,
                render_size,
                runtime_features,
                taa_history_enabled,
                screen_space_reflection_history_enabled,
                hzb_history_enabled,
                exposure_history_enabled,
                volumetric_history_quality,
            );
            history_initialization_needs_abort_cleanup =
                history_initialization_command_buffer.is_some();
            if taa_history_allocation_changed {
                self.core
                    .post_process
                    .invalidate_taa_resolve_bind_group_cache();
            }
            let target = require_offscreen_target_mut(self.target.as_mut())?;
            let parallel_record_min_passes_per_bucket = self.parallel_record_min_passes_per_bucket;
            let hzb_diagnostics_readback_enabled = self.hzb_diagnostics_readback_enabled;
            let (core, gpu_pass_timer, gpu_pipeline_statistics_timer) = (
                &mut self.core,
                self.gpu_pass_timer.as_mut(),
                self.gpu_pipeline_statistics_timer.as_mut(),
            );
            core.render_compiled_scene(
                &self.backend,
                &mut self.streamer,
                frame,
                target,
                pipeline,
                capabilities,
                &self.render_pass_executors,
                runtime_features,
                history_textures,
                history_frame_transaction,
                history_initialization_command_buffer,
                frame_generation,
                &mut submission_transaction,
                gpu_pass_timer,
                gpu_pipeline_statistics_timer,
                task_pool,
                parallel_record_min_passes_per_bucket,
                hzb_diagnostics_readback_enabled,
                viewport_capture,
                environment_ibl_bake_reservation,
                viewport_product_copy.as_ref(),
                surface_frame,
            )
        })();
        let runtime_outputs = match runtime_outputs_result {
            Ok(outputs) => outputs,
            Err(source) => {
                let scene_submission_was_accepted = matches!(
                    &source,
                    GraphicsError::FrameFailedAfterSceneSubmission { .. }
                );
                if history_initialization_needs_abort_cleanup && !scene_submission_was_accepted {
                    if let Some(handle) = history_input.current() {
                        self.history_targets.remove(&handle);
                    }
                }
                return Err(settle_failed_frame_submissions(
                    &self.backend,
                    &mut self.streamer,
                    submission_transaction,
                    source,
                ));
            }
        };
        self.last_gpu_timing_status = render_gpu_timing_status(
            self.gpu_pass_timing_requested,
            self.gpu_pass_timer.is_some(),
            self.gpu_pass_timer
                .as_ref()
                .and_then(GpuPassTimer::last_frame_observation),
        );
        let scene_submission = runtime_outputs.scene_submission();
        if let Err(source) = submission_transaction.validate_scene_submission(scene_submission) {
            return Err(settle_failed_frame_submissions(
                &self.backend,
                &mut self.streamer,
                submission_transaction,
                GraphicsError::FrameFailedAfterSceneSubmission {
                    scene_submission,
                    source: Box::new(source.into()),
                },
            ));
        }
        if let Err(source) = store_last_runtime_outputs(self, runtime_outputs) {
            return Err(settle_failed_frame_submissions(
                &self.backend,
                &mut self.streamer,
                submission_transaction,
                GraphicsError::FrameFailedAfterSceneSubmission {
                    scene_submission,
                    source: Box::new(source),
                },
            ));
        }
        let mut submission_receipt = submission_transaction.finish(scene_submission)?;
        submission_receipt = submission_receipt.with_submission_metrics(
            self.backend.frame_submission_metrics_since(
                submission_metrics_baseline,
                submission_receipt.logical_packet_count(),
            ),
        );
        self.scene_submission_completion_journal
            .track(frame_generation, scene_submission);
        let viewport_product_copy = match viewport_product_copy
            .map(|target| target.complete(scene_submission))
            .transpose()
        {
            Ok(copy) => copy,
            Err(source) => {
                return Err(GraphicsError::FrameProductPublicationFailed {
                    receipt: submission_receipt,
                    product_submission: Some(scene_submission),
                    source: Box::new(source.into()),
                });
            }
        };
        if viewport_product_copy.is_some() {
            let scene_receipt = submission_receipt.clone();
            submission_receipt = submission_receipt
                .with_viewport_product_submission(scene_submission)
                .map_err(|source| GraphicsError::FrameProductPublicationFailed {
                    receipt: scene_receipt,
                    product_submission: Some(scene_submission),
                    source: Box::new(source.into()),
                })?;
        }
        self.last_capture_target = Some(SceneRendererCaptureTarget {
            output_target: frame.output_target(),
            owns_final_target_output: frame
                .camera_stack_output_policy()
                .owns_final_target_output(),
        });
        self.generation += 1;
        debug_assert_eq!(submission_receipt.frame_generation(), self.generation);
        self.last_frame_submission_receipt = Some(submission_receipt.clone());
        Ok((
            submission_receipt,
            capture_admission
                .as_deref()
                .is_some_and(capture_request_was_admitted),
            viewport_product_copy,
        ))
    }
}

fn pipeline_writes_screen_space_reflection_history(pipeline: &CompiledRenderPipeline) -> bool {
    pipeline.writes_resource(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY)
}

#[cfg(test)]
mod tests {
    #[test]
    fn compiled_frame_tracks_scene_submission_after_finishing_the_receipt() {
        let source = include_str!("frame_submission_owner.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("compiled frame owner must retain a test boundary");
        let finish = production
            .find("submission_transaction.finish(scene_submission)")
            .expect("compiled frame must finish its submission receipt");
        let track = production
            .find(".track(frame_generation, scene_submission)")
            .expect("compiled frame must track scene completion");

        assert!(finish < track);
        assert_eq!(
            production
                .matches("self.poll_frame_submission_completions()?")
                .count(),
            1
        );
        assert!(production.contains("drain_pipeline_creation_diagnostics();"));
        assert!(!production.contains("drain_pipeline_creation_diagnostics(&self.backend.device)"));
    }

    #[test]
    fn pre_submit_failure_discards_history_whose_clear_never_reached_the_scene_packet() {
        let source = include_str!("frame_submission_owner.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("frame submission owner test boundary");
        let prepared = production
            .find("history_initialization_command_buffer.is_some()")
            .expect("history clear preparation receipt");
        let render = production
            .find("core.render_compiled_scene(")
            .expect("compiled scene boundary");
        let failure = production
            .find("if history_initialization_needs_abort_cleanup")
            .expect("pre-submit history cleanup");
        let remove = production[failure..]
            .find("self.history_targets.remove(&handle)")
            .map(|offset| failure + offset)
            .expect("unsubmitted history must be removed");

        assert!(prepared < render);
        assert!(render < failure);
        assert!(failure < remove);
        assert!(production.contains("GraphicsError::FrameFailedAfterSceneSubmission"));
    }
}
