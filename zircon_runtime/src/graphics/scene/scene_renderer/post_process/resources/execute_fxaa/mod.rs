use super::super::scene_post_process_resources::ScenePostProcessResources;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::scene::scene_renderer::post_process::resources::render_region::{
    apply_physical_render_region_to_pass, create_physical_terminal_region_params_buffer,
};
use crate::graphics::types::ViewportRenderRegion;
use crate::render_graph::RenderGraphAttachmentOps;

impl ScenePostProcessResources {
    pub(in crate::graphics::scene::scene_renderer) fn execute_fxaa(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        terminal_input_view: &wgpu::TextureView,
        final_color_view: &wgpu::TextureView,
        attachment_ops: RenderGraphAttachmentOps,
        render_region: ViewportRenderRegion,
    ) {
        let terminal_region_params_buffer = create_physical_terminal_region_params_buffer(
            device,
            "zircon-fxaa-terminal-region-params",
            render_region,
        );
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-fxaa-bind-group"),
            layout: &self.output_transfer_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(terminal_input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: terminal_region_params_buffer.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("FxaaPass"),
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
        if !apply_physical_render_region_to_pass(&mut pass, render_region) {
            return;
        }
        pass.set_pipeline(&self.fxaa_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
