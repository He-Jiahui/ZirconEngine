use std::time::Instant;

use crate::core::framework::render::{
    RenderCaptureReport, RenderFrameSubmissionReceipt, RenderFrameSubmissionTransaction,
};
use crate::graphics::backend::ViewportSurface;
use crate::graphics::types::{GraphicsError, ViewportFrame, ViewportRenderFrame};
use crate::rhi::SubmissionPollReceipt;
use zr_rhi_wgpu::WgpuNativeSurfaceFrameTarget;

use super::super::scene_renderer::{
    SceneRenderer, SceneRendererCaptureTarget, SceneRendererFrameTimingReport,
};
use super::super::scene_renderer_completion::route_frame_submission_completion_consumers;
use super::super::scene_renderer_render_with_pipeline::{
    ViewportAsyncCaptureSubmission, render_gpu_timing_status,
};
use super::super::scene_renderer_runtime_outputs::reset_last_runtime_outputs;
use super::super::scene_renderer_submission_failure::settle_failed_frame_submissions;
use super::super::scene_renderer_target::{
    ensure_offscreen_target, finish_viewport_frame, require_offscreen_target,
};
use super::super::target_extent::viewport_size;

impl SceneRenderer {
    pub(crate) fn render_frame_direct_submission(
        &mut self,
        frame: &ViewportRenderFrame,
        viewport_product_requested: bool,
    ) -> Result<ViewportAsyncCaptureSubmission, GraphicsError> {
        let (submission_receipt, viewport_product_copy) = self
            .render_frame_to_offscreen_target_with_viewport_product(
                frame,
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
            false,
        ))
    }

    pub(in crate::graphics) fn render_frame(
        &mut self,
        frame: &ViewportRenderFrame,
    ) -> Result<ViewportFrame, GraphicsError> {
        let capture_frame_timing = self.frame_timing_report_requested;
        let render_submission_started = capture_frame_timing.then(Instant::now);
        let (submission_receipt, _) =
            self.render_frame_to_offscreen_target_with_viewport_product(frame, false, None, None)?;
        let render_submission = render_submission_started.map(|started| started.elapsed());
        let readback_and_completion_started = capture_frame_timing.then(Instant::now);
        let viewport_frame = {
            let target = require_offscreen_target(self.target.as_ref())?;
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

            finish_viewport_frame(
                backend,
                &target.final_color,
                target.size,
                submission_receipt.frame_generation(),
                RenderCaptureReport::framework_offscreen(frame.output_target().kind(), target.size),
                &mut observe_poll,
            )?
        };
        if let (Some(render_submission), Some(readback_and_completion_started)) =
            (render_submission, readback_and_completion_started)
        {
            self.last_frame_timing_report = SceneRendererFrameTimingReport {
                render_submission,
                readback_and_completion: readback_and_completion_started.elapsed(),
            };
            self.frame_timing_report_requested = false;
        }
        Ok(viewport_frame)
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn render_frame_to_offscreen_target(
        &mut self,
        frame: &ViewportRenderFrame,
    ) -> Result<RenderFrameSubmissionReceipt, GraphicsError> {
        self.render_frame_to_offscreen_target_with_viewport_product(frame, false, None, None)
            .map(|(receipt, _)| receipt)
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn render_frame_to_offscreen_target_after_poll(
        &mut self,
        frame: &ViewportRenderFrame,
        poll_receipt: SubmissionPollReceipt,
        surface_frame: Option<(&ViewportSurface, &WgpuNativeSurfaceFrameTarget)>,
    ) -> Result<RenderFrameSubmissionReceipt, GraphicsError> {
        self.render_frame_to_offscreen_target_with_viewport_product(
            frame,
            false,
            surface_frame,
            Some(poll_receipt),
        )
        .map(|(receipt, _)| receipt)
    }

    fn render_frame_to_offscreen_target_with_viewport_product(
        &mut self,
        frame: &ViewportRenderFrame,
        viewport_product_requested: bool,
        surface_frame: Option<(&ViewportSurface, &WgpuNativeSurfaceFrameTarget)>,
        poll_receipt: Option<SubmissionPollReceipt>,
    ) -> Result<
        (
            RenderFrameSubmissionReceipt,
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
        let scene_submission_result = (|| -> Result<_, GraphicsError> {
            self.streamer.ensure_scene_resources(
                &self.backend,
                &self.backend.device,
                &self.core.texture_bind_group_layout,
                frame,
                &mut submission_transaction,
            )?;
            let output_target_frame_plan = self.streamer.output_target_frame_plan();

            let size = viewport_size(frame);
            if ensure_offscreen_target(&self.backend.device, &mut self.target, size, size) {
                self.core
                    .post_process
                    .invalidate_taa_resolve_bind_group_cache();
            }
            let target = require_offscreen_target(self.target.as_ref())?;
            let viewport_product_copy = viewport_product_requested
                .then(|| {
                    self.backend
                        .ui_surface_context()
                        .prepare_texture_for_external_image(
                            target.size.x,
                            target.size.y,
                            crate::graphics::scene::scene_renderer::FINAL_COLOR_FORMAT,
                            frame_generation,
                        )
                })
                .transpose()?;

            let (core, gpu_pass_timer) = (&mut self.core, self.gpu_pass_timer.as_mut());
            let scene_submission = core.render_scene(
                &self.backend,
                &mut self.streamer,
                frame,
                &target.scene_color_view,
                &target.final_color,
                &target.final_color_view,
                target.size,
                &target.depth_view,
                gpu_pass_timer,
                frame_generation,
                &mut submission_transaction,
                viewport_product_copy.as_ref(),
                surface_frame,
                output_target_frame_plan,
            )?;
            Ok((scene_submission, viewport_product_copy))
        })();
        let (scene_submission, viewport_product_copy) = match scene_submission_result {
            Ok(submission) => submission,
            Err(source) => {
                return Err(settle_failed_frame_submissions(
                    &self.backend,
                    &mut self.streamer,
                    submission_transaction,
                    source,
                ));
            }
        };
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
        self.last_gpu_timing_status = render_gpu_timing_status(
            self.gpu_pass_timing_requested,
            self.gpu_pass_timer.is_some(),
            self.gpu_pass_timer
                .as_ref()
                .and_then(crate::graphics::backend::GpuPassTimer::last_frame_observation),
        );
        self.last_capture_target = Some(SceneRendererCaptureTarget {
            output_target: frame.output_target(),
            owns_final_target_output: frame
                .camera_stack_output_policy()
                .owns_final_target_output(),
        });
        self.generation += 1;
        debug_assert_eq!(submission_receipt.frame_generation(), self.generation);
        self.last_frame_submission_receipt = Some(submission_receipt.clone());
        Ok((submission_receipt, viewport_product_copy))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn direct_frame_retains_the_device_qualified_submission_receipt() {
        let source = include_str!("render_frame.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("direct frame owner must retain a test boundary");

        assert!(source.contains("let (scene_submission, viewport_product_copy) = match"));
        assert!(source.contains("core.render_scene("));
        assert!(source.contains("validate_scene_submission(scene_submission)"));
        assert!(source.contains("submission_transaction.finish(scene_submission)"));
        assert!(production.contains(".track(frame_generation, scene_submission)"));
        assert!(
            source
                .contains("self.last_frame_submission_receipt = Some(submission_receipt.clone());")
        );
        assert!(source.contains("Ok((submission_receipt, viewport_product_copy))"));
    }

    #[test]
    fn direct_frame_owner_polls_before_resource_preparation_and_core_recording() {
        let source = include_str!("render_frame.rs");
        let poll = source
            .find("self.poll_frame_submission_completions()?")
            .expect("frame owner must pump completion");
        let transaction = source
            .find("RenderFrameSubmissionTransaction::begin(frame_generation, poll_receipt)")
            .expect("frame owner must begin the submission ledger");
        let ensure = source
            .find("self.streamer.ensure_scene_resources(")
            .expect("frame owner must prepare scene resources");
        let output_plan = source
            .find("let output_target_frame_plan = self.streamer.output_target_frame_plan();")
            .expect("frame owner must capture the resolved output plan");
        let render = source
            .find("core.render_scene(")
            .expect("frame owner must invoke core recording");

        assert!(poll < transaction);
        assert!(transaction < ensure);
        assert!(ensure < output_plan);
        assert!(output_plan < render);
        assert!(ensure < render);
    }

    #[test]
    fn direct_frame_publishes_one_submission_metrics_interval() {
        let source = include_str!("render_frame.rs");
        let baseline = source
            .find("let submission_metrics_baseline = self.backend.submission_metrics();")
            .expect("direct frame must sample after its completion poll");
        let ensure = source
            .find("self.streamer.ensure_scene_resources(")
            .expect("direct frame resource preparation");
        let finish = source
            .find("submission_transaction.finish(scene_submission)")
            .expect("direct frame receipt finalization");
        let attach = source
            .find(".with_submission_metrics(")
            .expect("direct frame metrics publication");

        assert!(baseline < ensure);
        assert!(ensure < finish);
        assert!(finish < attach);
        assert_eq!(source.matches("frame_submission_metrics_since(").count(), 1);
    }

    #[test]
    fn direct_frame_failure_settles_recorded_texture_submissions() {
        let source = include_str!("render_frame.rs");

        assert!(source.contains("&mut submission_transaction"));
        assert!(source.contains("let scene_submission_result = (||"));
        assert!(source.contains("settle_failed_frame_submissions("));
    }

    #[test]
    fn direct_viewport_product_is_prepared_before_recording_and_completed_with_scene_ticket() {
        let source = include_str!("render_frame.rs");
        let prepare = source
            .find(".prepare_texture_for_external_image(")
            .expect("product target must exist before scene recording");
        let render = source
            .find("core.render_scene(")
            .expect("direct scene recording");
        let complete = source
            .find("target.complete(scene_submission)")
            .expect("product target must be completed with the scene ticket");
        let attach = source
            .find(".with_viewport_product_submission(scene_submission)")
            .expect("frame receipt must retain the shared scene ticket");

        assert!(prepare < render);
        assert!(render < complete);
        assert!(complete < attach);
    }

    #[test]
    fn frame_timing_clocks_are_only_read_after_an_explicit_request() {
        let source = include_str!("render_frame.rs");

        assert!(source.contains("let capture_frame_timing = self.frame_timing_report_requested;"));
        assert!(
            source.contains(
                "let render_submission_started = capture_frame_timing.then(Instant::now);"
            )
        );
        assert!(source.contains(
            "let readback_and_completion_started = capture_frame_timing.then(Instant::now);"
        ));
        assert!(source.contains(
            "let render_submission = render_submission_started.map(|started| started.elapsed());"
        ));
        assert!(
            source.contains(
                "if let (Some(render_submission), Some(readback_and_completion_started))"
            )
        );
        assert!(source.contains("self.frame_timing_report_requested = false;"));

        let submission_end = source
            .find("let render_submission = render_submission_started.map")
            .expect("submission interval must end before readback begins");
        let readback_start = source
            .find("let readback_and_completion_started = capture_frame_timing.then(Instant::now);")
            .expect("readback interval must have an explicit start");
        assert!(
            submission_end < readback_start,
            "submission and readback timing intervals must not overlap"
        );
    }

    #[test]
    fn scene_completion_owner_routes_every_poll_before_draining_timer_results() {
        let source = include_str!("../scene_renderer_completion.rs");
        let direct_source = include_str!("render_frame.rs");
        let direct_caller = direct_source
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("direct frame caller must retain a test boundary");
        let compiled_caller = include_str!(
            "../scene_renderer_render_with_pipeline/render_frame_with_pipeline/frame_submission_owner.rs"
        );
        let production = source
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("scene completion owner must retain a test boundary");
        let poll = production
            .find("self.backend.poll_submission_completions()")
            .expect("scene renderer must poll its sole submission owner");
        let typed_query_route = production
            .find("backend.drain_product_diagnostic_query_results()")
            .expect("scene renderer must route completed typed query frames");
        let ibl_artifact_delivery = production
            .find("core.ibl_bake_runtime_writebacks")
            .expect("IBL artifact callbacks must drain after a completion poll");
        let timer_drain = production
            .find("GpuPassTimer::try_collect")
            .expect("scene renderer must drain timestamp results after polling");
        let statistics_drain = production
            .find("GpuPipelineStatisticsTimer::try_collect")
            .expect("scene renderer must drain statistics results after polling");

        assert_eq!(
            production.matches("poll_submission_completions()").count(),
            1
        );
        assert!(poll < ibl_artifact_delivery);
        assert!(ibl_artifact_delivery < typed_query_route);
        assert!(poll < typed_query_route);
        assert!(typed_query_route < timer_drain);
        assert!(typed_query_route < statistics_drain);
        assert!(poll < timer_drain);
        assert!(poll < statistics_drain);
        for caller in [direct_caller, compiled_caller] {
            assert!(caller.contains("self.poll_frame_submission_completions()?"));
            assert!(!caller.contains("self.backend.poll_submission_completions()?"));
            assert!(!caller.contains("readback_queue.poll_completed()"));
        }

        let readback_owner = include_str!(
            "../scene_renderer_render_with_pipeline/render_frame_with_pipeline/readback.rs"
        );
        let backend_submission =
            include_str!("../../../../backend/render_backend/render_backend_submission.rs");
        let backend_diagnostics =
            include_str!("../../../../backend/render_backend/render_backend_diagnostics.rs");
        assert!(readback_owner.contains("self.poll_frame_submission_completions()?"));
        assert!(!readback_owner.contains("self.backend.poll_submission_completions()?"));
        for backend_loop in [backend_submission, backend_diagnostics] {
            let poll = backend_loop
                .find("let poll_receipt = self.poll_submission_completions()?")
                .expect("explicit completion loop must poll through the backend owner");
            let route = backend_loop
                .find("observe_poll(poll_receipt)?")
                .expect("every explicit poll must be routed to SceneRenderer consumers");
            assert!(poll < route);
        }
    }
}
