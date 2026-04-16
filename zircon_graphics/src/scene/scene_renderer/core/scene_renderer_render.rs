use zircon_scene::RenderSceneSnapshot;

use crate::types::{EditorOrRuntimeFrame, GraphicsError, ViewportFrame, ViewportState};

use super::scene_renderer::SceneRenderer;
use super::scene_renderer_target::{ensure_offscreen_target, finish_viewport_frame};
use super::target_extent::viewport_size;

impl SceneRenderer {
    pub fn render(
        &mut self,
        snapshot: RenderSceneSnapshot,
        viewport: ViewportState,
    ) -> Result<ViewportFrame, GraphicsError> {
        self.render_frame(&EditorOrRuntimeFrame::from_snapshot(snapshot, viewport))
    }

    pub fn render_frame(
        &mut self,
        frame: &EditorOrRuntimeFrame,
    ) -> Result<ViewportFrame, GraphicsError> {
        let SceneRenderer {
            backend,
            core,
            streamer,
            target,
            generation,
            last_hybrid_gi_gpu_readback,
            last_virtual_geometry_gpu_readback,
            ..
        } = self;

        *last_hybrid_gi_gpu_readback = None;
        *last_virtual_geometry_gpu_readback = None;

        streamer.ensure_scene_resources(
            &backend.device,
            &backend.queue,
            &core.texture_bind_group_layout,
            frame,
        )?;

        let size = viewport_size(frame);
        ensure_offscreen_target(&backend.device, target, size);
        let target = target.as_ref().expect("offscreen target");

        core.render_scene(
            &backend.device,
            &backend.queue,
            streamer,
            frame,
            &target.final_color_view,
            &target.depth_view,
        )?;
        *generation += 1;

        finish_viewport_frame(&backend.device, &backend.queue, target, *generation)
    }
}
