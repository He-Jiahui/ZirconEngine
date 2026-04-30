use super::super::super::mesh::MeshDraw;
use super::normal_prepass_pipeline::NormalPrepassPipeline;

impl NormalPrepassPipeline {
    pub(crate) fn record<'a, I>(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        normal_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        mesh_draws: I,
    ) where
        I: IntoIterator<Item = &'a MeshDraw>,
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("NormalPrepass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: normal_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
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
