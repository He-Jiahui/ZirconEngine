use zircon_render_server::FrameHistoryHandle;

use crate::types::{EditorOrRuntimeFrame, GraphicsError, ViewportFrame};
use crate::CompiledRenderPipeline;

use super::runtime_features::runtime_features_from_pipeline;
use super::scene_renderer::SceneRenderer;
use super::scene_renderer_history::prepare_history_textures;
use super::scene_renderer_target::{ensure_offscreen_target, finish_viewport_frame};
use super::target_extent::viewport_size;

impl SceneRenderer {
    pub(crate) fn render_frame_with_pipeline(
        &mut self,
        frame: &EditorOrRuntimeFrame,
        pipeline: &CompiledRenderPipeline,
        history_handle: Option<FrameHistoryHandle>,
    ) -> Result<ViewportFrame, GraphicsError> {
        let SceneRenderer {
            backend,
            core,
            streamer,
            target,
            history_targets,
            generation,
            last_hybrid_gi_gpu_readback,
            last_virtual_geometry_gpu_readback,
        } = self;

        streamer.ensure_scene_resources(
            &backend.device,
            &backend.queue,
            &core.texture_bind_group_layout,
            frame,
        )?;

        let size = viewport_size(frame);
        ensure_offscreen_target(&backend.device, target, size);
        let target = target.as_mut().expect("offscreen target");
        let runtime_features = runtime_features_from_pipeline(pipeline);
        let (history_textures, history_available) = prepare_history_textures(
            &backend.device,
            &backend.queue,
            history_targets,
            history_handle,
            size,
            runtime_features,
        );

        let (hybrid_gi_gpu_readback, virtual_geometry_gpu_readback) = core.render_compiled_scene(
            &backend.device,
            &backend.queue,
            streamer,
            frame,
            target,
            runtime_features,
            history_textures,
            history_available,
        )?;
        *last_hybrid_gi_gpu_readback = hybrid_gi_gpu_readback
            .map(|pending| pending.collect(&backend.device))
            .transpose()?;
        *last_virtual_geometry_gpu_readback = virtual_geometry_gpu_readback
            .map(|pending| pending.collect(&backend.device))
            .transpose()?;
        *generation += 1;

        finish_viewport_frame(&backend.device, &backend.queue, target, *generation)
    }
}
