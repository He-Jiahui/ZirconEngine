use super::super::super::mesh::{MeshDraw, MeshPipelineCache};
use super::super::PreparedOverlayBuffers;
use super::viewport_overlay_renderer::ViewportOverlayRenderer;
use crate::scene::resources::ResourceStreamer;
use crate::types::EditorOrRuntimeFrame;

impl ViewportOverlayRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(
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
        prepared: &PreparedOverlayBuffers,
    ) {
        self.record_scene_content(
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
        self.record_overlays(
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            frame,
            prepared,
        );
    }
}
