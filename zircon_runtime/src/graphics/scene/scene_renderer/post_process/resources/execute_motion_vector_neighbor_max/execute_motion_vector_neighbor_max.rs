use crate::render_graph::RenderGraphAttachmentOps;

use super::super::super::clear_render_target::clear_render_target;
use super::super::super::scene_post_process_resources::ScenePostProcessResources;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;

impl ScenePostProcessResources {
    pub(crate) fn execute_motion_vector_neighbor_max(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        motion_vector_tile_max_coarse_view: &wgpu::TextureView,
        motion_vector_neighbor_max_view: &wgpu::TextureView,
        attachment_ops: RenderGraphAttachmentOps,
        enabled: bool,
    ) {
        if !enabled {
            clear_render_target(
                encoder,
                "ClearMotionVectorNeighborMaxPass",
                motion_vector_neighbor_max_view,
                wgpu::Color::BLACK,
            );
            return;
        }

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-motion-vector-neighbor-max-bind-group"),
            layout: &self.motion_vector_neighbor_max_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(motion_vector_tile_max_coarse_view),
            }],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("MotionVectorNeighborMaxPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: motion_vector_neighbor_max_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::BLACK),
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.motion_vector_neighbor_max_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
