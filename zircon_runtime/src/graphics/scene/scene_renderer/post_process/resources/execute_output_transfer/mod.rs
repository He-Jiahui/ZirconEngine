use super::super::scene_post_process_resources::ScenePostProcessResources;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::render_graph::RenderGraphAttachmentOps;

impl ScenePostProcessResources {
    pub(in crate::graphics::scene::scene_renderer) fn execute_output_transfer(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        tonemapped_view: &wgpu::TextureView,
        final_color_view: &wgpu::TextureView,
        attachment_ops: RenderGraphAttachmentOps,
    ) {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-output-transfer-bind-group"),
            layout: &self.output_transfer_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(tonemapped_view),
            }],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("OutputTransferPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: final_color_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::BLACK),
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.output_transfer_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
