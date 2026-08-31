use crate::core::framework::render::{
    CapturedHdrFrame, RenderCaptureReport, decode_rgba16f_texels,
};
use crate::graphics::types::{GraphicsError, ViewportFrame};

use super::super::super::scene_renderer::{SceneRenderer, SceneRendererCaptureTarget};
use super::super::super::scene_renderer_completion::route_frame_submission_completion_consumers;
use super::super::super::scene_renderer_target::finish_viewport_frame;
use super::output_target_capture_resource;

impl SceneRenderer {
    pub(crate) fn last_readback_poll_stats(&self) -> crate::graphics::backend::ReadbackPollStats {
        let metrics = self.backend.product_diagnostic_readback_metrics();
        crate::graphics::backend::ReadbackPollStats {
            completed_request_count: usize::try_from(metrics.succeeded_request_count())
                .unwrap_or(usize::MAX),
            completed_bytes: metrics.succeeded_bytes(),
            in_flight_count: metrics.in_flight_request_count(),
            in_flight_bytes: metrics.in_flight_bytes(),
            slot_reuse_rejection_count: u32::try_from(metrics.rejected_request_count())
                .unwrap_or(u32::MAX),
        }
    }

    pub(crate) fn capture_latest_frame(&mut self) -> Result<Option<ViewportFrame>, GraphicsError> {
        let Some(target) = self.target.as_ref() else {
            return Ok(None);
        };
        let capture_target = self
            .last_capture_target
            .unwrap_or(SceneRendererCaptureTarget {
                output_target: Default::default(),
                owns_final_target_output: true,
            });
        let output_target_capture = output_target_capture_resource(
            &self.streamer,
            capture_target.output_target,
            capture_target.owns_final_target_output,
        );
        let generation = self.generation;
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
                generation,
                capture_report,
                &mut observe_poll,
            )
            .map(Some);
        }
        finish_viewport_frame(
            backend,
            &target.final_color,
            target.size,
            generation,
            RenderCaptureReport::framework_offscreen(
                capture_target.output_target.kind(),
                target.size,
            ),
            &mut observe_poll,
        )
        .map(Some)
    }

    /// Reads the retained HDR scene color that the compiled frame already produced.
    pub(crate) fn capture_latest_scene_color_hdr(
        &mut self,
    ) -> Result<Option<CapturedHdrFrame>, GraphicsError> {
        let Some(target) = self.target.as_ref() else {
            return Ok(None);
        };
        let capture_target = self
            .last_capture_target
            .unwrap_or(SceneRendererCaptureTarget {
                output_target: Default::default(),
                owns_final_target_output: true,
            });
        let size = target.render_size;
        let generation = self.generation;
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
        let bytes = backend.read_product_diagnostic_texture_rgba16float_blocking(
            generation,
            &target.scene_color,
            0,
            0,
            size.x,
            size.y,
            "zircon-compiled-scene-color-hdr-capture",
            &mut observe_poll,
        )?;

        Ok(Some(CapturedHdrFrame::with_capture_report(
            size.x,
            size.y,
            decode_rgba16f_texels(&bytes),
            generation,
            RenderCaptureReport::framework_offscreen(capture_target.output_target.kind(), size),
        )))
    }

    pub(crate) fn poll_readback_completions(&mut self) -> Result<(), GraphicsError> {
        self.poll_frame_submission_completions()?;
        Ok(())
    }

    pub(crate) fn wait_for_readback_completions(&mut self) -> Result<(), GraphicsError> {
        let backend = &self.backend;
        let observe_poll = |poll_receipt| {
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
        backend.wait_for_product_diagnostic_deliveries(observe_poll)
    }
}
