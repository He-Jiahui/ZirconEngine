use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshSceneDataBindHandle;
use crate::graphics::scene::scene_renderer::mesh::{MeshDraw, MeshPipelineCache};
use crate::graphics::scene::scene_renderer::overlay::ViewportOverlayRenderer;
use crate::graphics::types::ViewportRenderFrame;

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
        gpu_scene_bind_group: Option<MeshSceneDataBindHandle<'_>>,
        mesh_pipelines: &mut MeshPipelineCache,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
    ) {
        self.record_preview_sky(encoder, color_view, depth_view, scene_bind_group, frame);
        self.record_meshes(
            encoder,
            device,
            color_view,
            depth_view,
            scene_bind_group,
            mesh_draws,
            gpu_scene_bind_group,
            mesh_pipelines,
            streamer,
            frame,
        );
    }
}
