use crate::scene::resources::ResourceStreamer;
use crate::scene::scene_renderer::mesh::{MeshDraw, MeshPipelineCache};
use crate::scene::scene_renderer::overlay::ViewportOverlayRenderer;
use crate::types::EditorOrRuntimeFrame;

impl ViewportOverlayRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_scene_content(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        mesh_draws: &[MeshDraw],
        mesh_pipelines: &mut MeshPipelineCache,
        streamer: &ResourceStreamer,
        frame: &EditorOrRuntimeFrame,
    ) {
        self.record_preview_sky(encoder, color_view, depth_view, scene_bind_group, frame);
        self.record_meshes(
            encoder,
            device,
            color_view,
            depth_view,
            scene_bind_group,
            mesh_draws.iter(),
            mesh_pipelines,
            streamer,
            frame,
        );
    }
}
