use crate::types::{EditorOrRuntimeFrame, GraphicsError, ViewportFrame};

use super::super::scene_renderer::SceneRenderer;
use super::super::scene_renderer_runtime_outputs::reset_last_runtime_outputs;
use super::super::scene_renderer_target::{ensure_offscreen_target, finish_viewport_frame};
use super::super::target_extent::viewport_size;

impl SceneRenderer {
    pub fn render_frame(
        &mut self,
        frame: &EditorOrRuntimeFrame,
    ) -> Result<ViewportFrame, GraphicsError> {
        reset_last_runtime_outputs(self);

        self.streamer.ensure_scene_resources(
            &self.backend.device,
            &self.backend.queue,
            &self.core.texture_bind_group_layout,
            frame,
        )?;

        let size = viewport_size(frame);
        ensure_offscreen_target(&self.backend.device, &mut self.target, size);
        let target = self.target.as_ref().expect("offscreen target");

        self.core.render_scene(
            &self.backend.device,
            &self.backend.queue,
            &self.streamer,
            frame,
            &target.final_color_view,
            &target.depth_view,
        )?;
        self.generation += 1;

        finish_viewport_frame(
            &self.backend.device,
            &self.backend.queue,
            target,
            self.generation,
        )
    }
}
