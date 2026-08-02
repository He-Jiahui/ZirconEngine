use crate::render_graph::RenderGraphAttachmentOps;

use super::super::super::clear_render_target::clear_render_target;
use super::super::super::scene_post_process_resources::ScenePostProcessResources;
use crate::core::framework::render::{FULLSCREEN_PARAMS_BINDING, FULLSCREEN_PASS_INPUT_GROUP};
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::shader::{motion_vector_tile_max_pass_plan, MOTION_VECTOR_SOURCE_RESOURCE};

impl ScenePostProcessResources {
    pub(crate) fn execute_motion_vector_tile_max(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        motion_vector_source_view: &wgpu::TextureView,
        motion_vector_tile_max_view: &wgpu::TextureView,
        attachment_ops: RenderGraphAttachmentOps,
        enabled: bool,
    ) {
        if !enabled {
            clear_render_target(
                encoder,
                "ClearMotionVectorTileMaxPass",
                motion_vector_tile_max_view,
                wgpu::Color::BLACK,
            );
            return;
        }

        let pass_plan = motion_vector_tile_max_pass_plan();
        let source_binding = pass_plan
            .resource_binding(MOTION_VECTOR_SOURCE_RESOURCE)
            .expect("motion-vector fullscreen source binding must exist")
            .abi
            .binding;
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-motion-vector-tile-max-bind-group"),
            layout: &self.motion_vector_tile_max_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: source_binding,
                resource: wgpu::BindingResource::TextureView(motion_vector_source_view),
            }],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("MotionVectorTileMaxPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: motion_vector_tile_max_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::BLACK),
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.motion_vector_tile_max_pipeline);
        pass.set_bind_group(FULLSCREEN_PASS_INPUT_GROUP, &bind_group, &[]);
        pass.set_bind_group(
            FULLSCREEN_PARAMS_BINDING.group,
            self.motion_vector_tile_max_parameter_bindings.bind_group(),
            &[],
        );
        pass.draw(0..3, 0..1);
    }
}
