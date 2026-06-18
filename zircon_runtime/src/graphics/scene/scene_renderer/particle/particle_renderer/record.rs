use super::super::build_particle_vertices::build_particle_vertices;
use super::ParticleRenderer;
use crate::graphics::types::{ViewportRenderFrame, ViewportRenderRegion};
use wgpu::util::DeviceExt;

impl ParticleRenderer {
    pub(crate) fn record(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        frame: &ViewportRenderFrame,
        render_region: ViewportRenderRegion,
    ) {
        self.record_vertex_batch(
            device,
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            &build_particle_vertices(frame, true),
            &self.pipeline,
            render_region,
        );
        self.record_vertex_batch(
            device,
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            &build_particle_vertices(frame, false),
            &self.overlay_pipeline,
            render_region,
        );
    }

    fn record_vertex_batch(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        vertices: &[super::super::particle_vertex::ParticleVertex],
        pipeline: &wgpu::RenderPipeline,
        render_region: ViewportRenderRegion,
    ) {
        if vertices.is_empty() {
            return;
        }

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-particle-vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ParticlePass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
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
        if !render_region.apply_to_render_pass(&mut pass) {
            return;
        }
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, scene_bind_group, &[]);
        pass.set_vertex_buffer(0, buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }
}
