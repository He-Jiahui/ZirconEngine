use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshDrawCommandReplayer, MeshDrawCommandStream, MeshDrawReplayStats, MeshSceneDataBindHandle,
};
use crate::graphics::types::ViewportRenderRegion;
use crate::render_graph::RenderGraphAttachmentOps;

use super::DeferredSceneResources;

impl DeferredSceneResources {
    pub(crate) fn record_gbuffer_geometry<'a, I>(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        gbuffer_albedo_view: &wgpu::TextureView,
        gbuffer_material_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        gpu_scene_bind_group: Option<MeshSceneDataBindHandle<'a>>,
        albedo_attachment_ops: RenderGraphAttachmentOps,
        material_attachment_ops: RenderGraphAttachmentOps,
        render_region: ViewportRenderRegion,
        mesh_draw_commands: I,
    ) -> MeshDrawReplayStats
    where
        I: IntoIterator<Item = MeshDrawCommandStream<'a>>,
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("DeferredGeometryPass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: gbuffer_albedo_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: color_attachment_operations(
                        albedo_attachment_ops,
                        wgpu::Color::TRANSPARENT,
                    ),
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: gbuffer_material_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: color_attachment_operations(material_attachment_ops, wgpu::Color::BLACK),
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        if !render_region.apply_to_render_pass(&mut pass) {
            return MeshDrawReplayStats::default();
        }
        pass.set_pipeline(&self.geometry_pipeline);
        pass.set_bind_group(0, scene_bind_group, &[]);
        let mut replayer = MeshDrawCommandReplayer::default();
        for stream in mesh_draw_commands {
            replayer.replay_command_stream(&mut pass, stream, |replayer, pass, command| {
                replayer.bind_gpu_scene_if_needed(pass, command, gpu_scene_bind_group);
                replayer.bind_standard_material_if_needed(pass, command);
                replayer.bind_geometry_if_needed(pass, command);
                true
            });
        }
        replayer.stats()
    }
}
