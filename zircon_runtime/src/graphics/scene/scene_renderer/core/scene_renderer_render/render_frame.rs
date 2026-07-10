use crate::core::framework::render::RenderCaptureReport;
use crate::graphics::types::{GraphicsError, ViewportFrame, ViewportRenderFrame};

use super::super::scene_renderer::SceneRenderer;
use super::super::scene_renderer_runtime_outputs::reset_last_runtime_outputs;
use super::super::scene_renderer_target::{ensure_offscreen_target, finish_viewport_frame};
use super::super::target_extent::viewport_size;

impl SceneRenderer {
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

    pub fn render_frame(
        &mut self,
        frame: &ViewportRenderFrame,
    ) -> Result<ViewportFrame, GraphicsError> {
        self.render_frame_to_offscreen_target(frame)?;
        let target = self.target.as_ref().expect("offscreen target");

        finish_viewport_frame(
            &self.backend.device,
            &self.backend.queue,
            &target.final_color,
            target.size,
            self.generation,
            RenderCaptureReport::framework_offscreen(frame.output_target().kind(), target.size),
        )
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn render_frame_to_offscreen_target(
        &mut self,
        frame: &ViewportRenderFrame,
    ) -> Result<(), GraphicsError> {
        reset_last_runtime_outputs(self);

        self.streamer.ensure_scene_resources(
            &self.backend.device,
            &self.backend.queue,
            &self.core.texture_bind_group_layout,
            frame,
        )?;

        let size = viewport_size(frame);
        ensure_offscreen_target(&self.backend.device, &mut self.target, size, size);
        let target = self.target.as_ref().expect("offscreen target");

        self.core.render_scene(
            &self.backend.device,
            &self.backend.queue,
            &self.streamer,
            frame,
            &target.scene_color_view,
            &target.final_color_view,
            &target.depth_view,
        )?;
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
