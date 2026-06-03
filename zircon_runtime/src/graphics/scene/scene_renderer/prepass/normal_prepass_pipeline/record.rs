use super::super::super::attachment_ops::{
    color_attachment_operations, depth_attachment_operations,
};
use super::super::super::mesh::MeshDraw;
use super::normal_prepass_pipeline::NormalPrepassPipeline;
use crate::render_graph::RenderGraphAttachmentOps;

impl NormalPrepassPipeline {
    pub(crate) fn record_with_attachment_ops<'a, I>(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        normal_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        mesh_draws: I,
        normal_attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) where
        I: IntoIterator<Item = &'a MeshDraw>,
    {
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
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, scene_bind_group, &[]);
        for draw in mesh_draws {
            draw.bind_model(&mut pass);
            draw.bind_geometry_buffers(&mut pass);
            draw.record_indexed_draw(&mut pass);
        }
    }
}
