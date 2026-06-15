use crate::core::framework::render::AntiAliasSettings;
use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::scene::scene_renderer::post_process::{
    PostProcessDepthSamplingMode, ScenePostProcessResources,
};
use crate::render_graph::RenderGraphAttachmentOps;

use super::taa_resolve_params::TaaResolveParams;

impl ScenePostProcessResources {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn execute_taa_resolve(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        viewport_size: UVec2,
        scene_color_view: &wgpu::TextureView,
        scene_depth_view: &wgpu::TextureView,
        scene_velocity_view: &wgpu::TextureView,
        taa_history_previous_view: &wgpu::TextureView,
        taa_reactive_mask_view: &wgpu::TextureView,
        taa_output_view: &wgpu::TextureView,
        taa_history_current_view: &wgpu::TextureView,
        taa_output_attachment_ops: RenderGraphAttachmentOps,
        taa_history_attachment_ops: RenderGraphAttachmentOps,
        history_valid: bool,
        anti_alias: AntiAliasSettings,
    ) {
        let params = TaaResolveParams::new(
            viewport_size,
            anti_alias.mode == crate::core::framework::render::AntiAliasMode::Taa && history_valid,
            anti_alias.taa_quality,
        );
        queue.write_buffer(
            &self.taa_resolve_params_buffer,
            0,
            bytemuck::bytes_of(&params),
        );
        let scene_depth_binding_view = match self.depth_sampling_mode {
            PostProcessDepthSamplingMode::RawDepthTexture => scene_depth_view,
            PostProcessDepthSamplingMode::ViewportDepthFallback => &self.black_texture_view,
        };
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-taa-resolve-bind-group"),
            layout: &self.taa_resolve_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(scene_color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(scene_depth_binding_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(scene_velocity_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(taa_history_previous_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.taa_resolve_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(taa_reactive_mask_view),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("TaaResolvePass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: taa_output_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: color_attachment_operations(taa_output_attachment_ops, wgpu::Color::BLACK),
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: taa_history_current_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: color_attachment_operations(
                        taa_history_attachment_ops,
                        wgpu::Color::BLACK,
                    ),
                }),
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.taa_resolve_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }

    pub(crate) fn execute_taa_reactive_mask_clear(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        taa_reactive_mask_view: &wgpu::TextureView,
        attachment_ops: RenderGraphAttachmentOps,
    ) {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("TaaReactiveMaskClearPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: taa_reactive_mask_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::BLACK),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
}
