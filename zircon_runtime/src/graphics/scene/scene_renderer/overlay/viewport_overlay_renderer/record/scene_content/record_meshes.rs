use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshDrawCommandStream, MeshSceneDataBindHandle,
};
use crate::graphics::scene::scene_renderer::mesh::{
    build_mesh_pass_command_buffers, MeshDraw, MeshPipelineCache,
};
use crate::graphics::scene::scene_renderer::overlay::ViewportOverlayRenderer;
use crate::graphics::types::ViewportRenderFrame;

impl ViewportOverlayRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_meshes(
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
        let command_buffers = build_mesh_pass_command_buffers(mesh_draws, mesh_pipelines);
        let commands = [
            MeshDrawCommandStream::new(command_buffers.opaque().commands(), None),
            MeshDrawCommandStream::new(command_buffers.alpha_mask().commands(), None),
            MeshDrawCommandStream::new(command_buffers.transparent().commands(), None),
        ];
        let _replay_stats = self.base_scene.record_commands_with_attachment_ops(
            encoder,
            device,
            color_view,
            depth_view,
            scene_bind_group,
            gpu_scene_bind_group,
            commands,
            mesh_pipelines,
            streamer,
            frame,
            None,
            None,
            None,
            crate::render_graph::RenderGraphAttachmentOps::load_store(),
            crate::render_graph::RenderGraphAttachmentOps::load_store(),
        );
    }
}
