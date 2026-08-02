use super::super::scene_post_process_resources::ScenePostProcessResources;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::scene::scene_renderer::post_process::resources::render_region::apply_physical_render_region_to_pass;
use crate::graphics::types::ViewportRenderRegion;
use crate::render_graph::RenderGraphAttachmentOps;

impl ScenePostProcessResources {
    pub(in crate::graphics::scene::scene_renderer) fn execute_output_transfer(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        tonemapped_view: &wgpu::TextureView,
        final_color_view: &wgpu::TextureView,
        attachment_ops: RenderGraphAttachmentOps,
        render_region: ViewportRenderRegion,
    ) {
        let (terminal_resource_cache, output_transfer_bind_group_layout, output_transfer_pipeline) =
            match self {
                Self::Full(resources) => (
                    &resources.terminal_resource_cache,
                    &resources.output_transfer_bind_group_layout,
                    &resources.output_transfer_pipeline,
                ),
                Self::OutputTransferOnly(resources) => (
                    &resources.terminal_resource_cache,
                    &resources.bind_group_layout,
                    &resources.pipeline,
                ),
            };
        // FINAL_COMPOSITED is the full-resolution terminal input after upscale, so every
        // output-transfer target uses physical coordinates rather than the internal render size.
        let terminal_region_params_buffer =
            terminal_resource_cache.physical_terminal_region_params_buffer(device, render_region);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-output-transfer-bind-group"),
            layout: output_transfer_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(tonemapped_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: terminal_region_params_buffer.as_entire_binding(),
                },
            ],
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
        let region_applied = apply_physical_render_region_to_pass(&mut pass, render_region);
        if !region_applied {
            return;
        }
        pass.set_pipeline(output_transfer_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
