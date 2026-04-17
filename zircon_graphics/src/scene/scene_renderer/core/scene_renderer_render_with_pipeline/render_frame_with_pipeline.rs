use zircon_render_server::FrameHistoryHandle;

use crate::types::{EditorOrRuntimeFrame, GraphicsError, ViewportFrame};
use crate::CompiledRenderPipeline;

use super::super::runtime_features::runtime_features_from_pipeline;
use super::super::scene_renderer::SceneRenderer;
use super::super::scene_renderer_history::prepare_history_textures;
use super::super::scene_renderer_runtime_outputs::{
    reset_last_runtime_outputs, store_last_runtime_outputs,
};
use super::super::scene_renderer_target::{ensure_offscreen_target, finish_viewport_frame};
use super::super::target_extent::viewport_size;

impl SceneRenderer {
    pub(crate) fn render_frame_with_pipeline(
        &mut self,
        frame: &EditorOrRuntimeFrame,
        pipeline: &CompiledRenderPipeline,
        history_handle: Option<FrameHistoryHandle>,
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
        let runtime_features = runtime_features_from_pipeline(pipeline);

        let (
            hybrid_gi_gpu_readback,
            virtual_geometry_gpu_readback,
            indirect_draw_count,
            indirect_buffer_count,
            indirect_segment_count,
            indirect_args_buffer,
            indirect_args_count,
        ) = {
            let (history_textures, history_available) = prepare_history_textures(
                &self.backend.device,
                &self.backend.queue,
                &mut self.history_targets,
                history_handle,
                size,
                runtime_features,
            );
            let target = self.target.as_mut().expect("offscreen target");
            self.core.render_compiled_scene(
                &self.backend.device,
                &self.backend.queue,
                &self.streamer,
                frame,
                target,
                runtime_features,
                history_textures,
                history_available,
            )?
        };
        store_last_runtime_outputs(
            self,
            hybrid_gi_gpu_readback,
            virtual_geometry_gpu_readback,
            indirect_draw_count,
            indirect_buffer_count,
            indirect_segment_count,
            indirect_args_buffer,
            indirect_args_count,
        )?;
        self.generation += 1;

        let target = self.target.as_ref().expect("offscreen target");
        finish_viewport_frame(
            &self.backend.device,
            &self.backend.queue,
            target,
            self.generation,
        )
    }
}
