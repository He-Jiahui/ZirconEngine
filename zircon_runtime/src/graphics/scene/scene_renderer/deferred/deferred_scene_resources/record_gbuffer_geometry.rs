use super::super::super::mesh::MeshDraw;
use super::DeferredSceneResources;

impl DeferredSceneResources {
    pub(crate) fn record_gbuffer_geometry<'a, I>(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        gbuffer_albedo_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        mesh_draws: I,
    ) where
        I: IntoIterator<Item = &'a MeshDraw>,
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("DeferredGeometryPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: gbuffer_albedo_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
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
        pass.set_pipeline(&self.geometry_pipeline);
        pass.set_bind_group(0, scene_bind_group, &[]);
        for draw in mesh_draws {
            pass.set_bind_group(1, &draw.model_bind_group, &[]);
            pass.set_bind_group(2, &draw.texture.bind_group, &[]);
            pass.set_vertex_buffer(0, draw.mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(draw.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            if let Some(indirect_args_buffer) = &draw.indirect_args_buffer {
                pass.draw_indexed_indirect(indirect_args_buffer, draw.indirect_args_offset);
            } else {
                pass.draw_indexed(
                    draw.first_index..(draw.first_index + draw.draw_index_count),
                    0,
                    0..1,
                );
            }
        }
    }
}
