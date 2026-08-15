use std::time::Instant;

use crate::core::framework::render::RenderCaptureReport;
use crate::graphics::types::{GraphicsError, ViewportFrame, ViewportRenderFrame};

use super::super::scene_renderer::{SceneRenderer, SceneRendererFrameTimingReport};
use super::super::scene_renderer_render_with_pipeline::render_gpu_timing_status;
use super::super::scene_renderer_runtime_outputs::reset_last_runtime_outputs;
use super::super::scene_renderer_target::{ensure_offscreen_target, finish_viewport_frame};
use super::super::target_extent::viewport_size;

impl SceneRenderer {
    pub(in crate::graphics) fn render_frame(
        &mut self,
        frame: &ViewportRenderFrame,
    ) -> Result<ViewportFrame, GraphicsError> {
        let capture_frame_timing = self.frame_timing_report_requested;
        let render_submission_started = capture_frame_timing.then(Instant::now);
        self.render_frame_to_offscreen_target(frame)?;
        let render_submission = render_submission_started.map(|started| started.elapsed());
        let readback_and_completion_started = capture_frame_timing.then(Instant::now);
        let viewport_frame = {
            let target = self.target.as_ref().expect("offscreen target");

            finish_viewport_frame(
                &self.backend.device,
                &self.backend.queue,
                &target.final_color,
                target.size,
                self.generation,
                RenderCaptureReport::framework_offscreen(frame.output_target().kind(), target.size),
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
    ) -> Result<(), GraphicsError> {
        reset_last_runtime_outputs(self);

        self.last_gpu_timer_frame_result = {
            let (gpu_pass_timer, readback_queue) =
                (&mut self.gpu_pass_timer, &mut self.core.readback_queue);
            gpu_pass_timer
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
        if ensure_offscreen_target(&self.backend.device, &mut self.target, size, size) {
            self.core
                .post_process
                .invalidate_taa_resolve_bind_group_cache();
        }
        let target = self.target.as_ref().expect("offscreen target");

        let (core, gpu_pass_timer) = (&mut self.core, self.gpu_pass_timer.as_mut());
        core.render_scene(
            &self.backend.device,
            &self.backend.queue,
            &self.streamer,
            frame,
            &target.scene_color_view,
            &target.final_color_view,
            &target.depth_view,
            gpu_pass_timer,
            frame_generation,
        )?;
        self.last_gpu_timing_status = render_gpu_timing_status(
            self.gpu_pass_timing_requested,
            self.gpu_pass_timer.is_some(),
            self.gpu_pass_timer
                .as_ref()
                .and_then(crate::graphics::backend::GpuPassTimer::last_frame_observation),
        );
        self.streamer.execute_output_target_writeback(
            &self.backend.device,
            &self.backend.queue,
            frame,
            &target.final_color,
            &target.final_color_view,
            target.size,
        )?;
        self.generation += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn frame_timing_clocks_are_only_read_after_an_explicit_request() {
        let source = include_str!("render_frame.rs");

        assert!(source.contains("let capture_frame_timing = self.frame_timing_report_requested;"));
        assert!(source
            .contains("let render_submission_started = capture_frame_timing.then(Instant::now);"));
        assert!(source.contains(
            "let readback_and_completion_started = capture_frame_timing.then(Instant::now);"
        ));
        assert!(source.contains(
            "let render_submission = render_submission_started.map(|started| started.elapsed());"
        ));
        assert!(source
            .contains("if let (Some(render_submission), Some(readback_and_completion_started))"));
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
}
