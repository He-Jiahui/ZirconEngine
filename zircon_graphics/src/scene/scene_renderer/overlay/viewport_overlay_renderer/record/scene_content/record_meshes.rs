use crate::scene::resources::ResourceStreamer;
use crate::scene::scene_renderer::mesh::{MeshDraw, MeshPipelineCache};
use crate::scene::scene_renderer::overlay::ViewportOverlayRenderer;
use crate::types::EditorOrRuntimeFrame;

impl ViewportOverlayRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_meshes<'a, I>(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        mesh_draws: I,
        mesh_pipelines: &mut MeshPipelineCache,
        streamer: &ResourceStreamer,
        frame: &EditorOrRuntimeFrame,
    ) where
        I: IntoIterator<Item = &'a MeshDraw>,
    {
        self.base_scene.record(
            encoder,
            device,
            color_view,
            depth_view,
            scene_bind_group,
            mesh_draws,
            mesh_pipelines,
            streamer,
            frame,
        );
    }
}
