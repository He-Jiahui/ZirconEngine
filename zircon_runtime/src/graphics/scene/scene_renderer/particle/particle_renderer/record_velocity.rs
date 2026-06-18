use wgpu::util::DeviceExt;

use super::super::build_particle_velocity_vertices::build_particle_velocity_vertices;
use super::ParticleRenderer;
use crate::graphics::scene::scene_renderer::attachment_ops::{
    color_attachment_operations, depth_attachment_operations,
};
use crate::graphics::types::{ViewportRenderFrame, ViewportRenderRegion};
use crate::render_graph::RenderGraphAttachmentOps;

impl ParticleRenderer {
    pub(crate) fn record_velocity(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        pass_name: &str,
        velocity_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        frame: &ViewportRenderFrame,
        render_region: ViewportRenderRegion,
        attachment_ops: RenderGraphAttachmentOps,
    ) {
        let vertices = build_particle_velocity_vertices(frame);
        if vertices.is_empty() {
            return;
        }

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-particle-velocity-vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(pass_name),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: velocity_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::BLACK),
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(depth_attachment_operations(
                    RenderGraphAttachmentOps::load_store(),
                    1.0,
                )),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        if !render_region.apply_to_render_pass(&mut pass) {
            return;
        }
        pass.set_pipeline(&self.velocity_pipeline);
        pass.set_bind_group(0, scene_bind_group, &[]);
        pass.set_vertex_buffer(0, buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }
}
