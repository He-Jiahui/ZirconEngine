use crate::graphics::scene::scene_renderer::attachment_ops::{
    color_attachment_operations, depth_attachment_operations,
};
use crate::graphics::types::ViewportRenderRegion;
use crate::render_graph::RenderGraphAttachmentOps;

use super::super::scene_post_process_resources::ScenePostProcessResources;

impl ScenePostProcessResources {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics::scene::scene_renderer) fn execute_half_resolution_transparency_depth_downsample(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source_depth_view: &wgpu::TextureView,
        half_color_view: &wgpu::TextureView,
        half_depth_view: &wgpu::TextureView,
        render_region: ViewportRenderRegion,
        color_attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-halfres-transparency-depth-bind-group"),
            layout: &self.half_res_transparency_depth_downsample_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source_depth_view),
            }],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("HalfResolutionTransparencyDepthDownsamplePass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: half_color_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(color_attachment_ops, wgpu::Color::TRANSPARENT),
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: half_depth_view,
                depth_ops: Some(depth_attachment_operations(depth_attachment_ops, 1.0)),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        if !render_region.apply_physical_to_render_pass(&mut pass) {
            return;
        }
        pass.set_pipeline(&self.half_res_transparency_depth_downsample_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics::scene::scene_renderer) fn execute_half_resolution_transparency_composite(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        half_color_view: &wgpu::TextureView,
        half_depth_view: &wgpu::TextureView,
        full_depth_view: &wgpu::TextureView,
        scene_color_view: &wgpu::TextureView,
        render_region: ViewportRenderRegion,
        attachment_ops: RenderGraphAttachmentOps,
        depth_sigma: u16,
    ) {
        let params = [f32::from(depth_sigma), 0.0, 0.0, 0.0];
        queue.write_buffer(
            &self.half_res_transparency_params_buffer,
            0,
            bytemuck::cast_slice(&params),
        );
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-halfres-transparency-composite-bind-group"),
            layout: &self.half_res_transparency_composite_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(half_color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(half_depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(full_depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.half_res_transparency_params_buffer.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("HalfResolutionTransparencyCompositePass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: scene_color_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::TRANSPARENT),
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        if !render_region.apply_physical_to_render_pass(&mut pass) {
            return;
        }
        pass.set_pipeline(&self.half_res_transparency_composite_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn half_resolution_transparency_passes_keep_cached_pipelines() {
        let source = include_str!("execute_half_res_transparency.rs");

        assert!(source.contains("half_res_transparency_depth_downsample_pipeline"));
        assert!(source.contains("half_res_transparency_composite_pipeline"));
        assert!(source.contains("queue.write_buffer("));
        assert!(source.contains("binding: 4"));
        assert!(!source.contains("create_render_pipeline"));
    }
}
