use super::super::super::attachment_ops::{
    color_attachment_operations, depth_attachment_operations,
};
use super::super::super::mesh::mesh_pass::{
    MeshDrawCommandReplayer, MeshDrawCommandStream, MeshDrawReplayStats, MeshSceneDataBindHandle,
};
use super::normal_prepass_pipeline::NormalPrepassPipeline;
use crate::graphics::types::ViewportRenderRegion;
use crate::render_graph::RenderGraphAttachmentOps;

impl NormalPrepassPipeline {
    pub(crate) fn record_commands_with_attachment_ops<'a>(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        normal_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        gpu_scene_bind_group: Option<MeshSceneDataBindHandle<'a>>,
        mesh_draw_commands: MeshDrawCommandStream<'a>,
        render_region: ViewportRenderRegion,
        normal_attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> MeshDrawReplayStats {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("NormalPrepass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: normal_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(normal_attachment_ops, wgpu::Color::BLACK),
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(depth_attachment_operations(depth_attachment_ops, 1.0)),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        if !render_region.apply_to_render_pass(&mut pass) {
            return MeshDrawReplayStats::default();
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, scene_bind_group, &[]);
        let mut replayer = MeshDrawCommandReplayer::default();
        replayer.replay_command_stream(&mut pass, mesh_draw_commands, |replayer, pass, command| {
            replayer.bind_gpu_scene_if_needed(pass, command, gpu_scene_bind_group);
            replayer.bind_standard_material_if_needed(pass, command);
            replayer.bind_geometry_if_needed(pass, command);
            true
        });
        replayer.stats()
    }
}
