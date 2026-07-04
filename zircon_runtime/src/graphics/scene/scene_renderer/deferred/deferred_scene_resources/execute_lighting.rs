use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::scene::scene_renderer::shadow::atlas::{
    ShadowAtlasResources, SHADOW_ATLAS_BINDING, SHADOW_ATLAS_SAMPLER_BINDING,
    SHADOW_ATLAS_SLOT_BUFFER_BINDING, SHADOW_GLOBALS_BINDING,
};
use crate::graphics::types::ViewportRenderRegion;
use crate::render_graph::RenderGraphAttachmentOps;

use super::DeferredSceneResources;

impl DeferredSceneResources {
    pub(crate) fn execute_lighting(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        scene_bind_group: &wgpu::BindGroup,
        gpu_scene_bind_group: &wgpu::BindGroup,
        gbuffer_albedo_view: &wgpu::TextureView,
        normal_view: &wgpu::TextureView,
        gbuffer_material_view: &wgpu::TextureView,
        scene_depth_view: &wgpu::TextureView,
        shadow_atlas_resources: Option<&ShadowAtlasResources>,
        light_grid_params_buffer: &wgpu::Buffer,
        light_zbins_buffer: &wgpu::Buffer,
        light_tile_masks_buffer: &wgpu::Buffer,
        background_view: &wgpu::TextureView,
        scene_color_view: &wgpu::TextureView,
        attachment_ops: RenderGraphAttachmentOps,
        render_region: ViewportRenderRegion,
    ) {
        let shadow_atlas_view = shadow_atlas_resources
            .map(ShadowAtlasResources::atlas_view)
            .unwrap_or(&self.shadow_atlas_fallback_view);
        let shadow_atlas_sampler = shadow_atlas_resources
            .map(ShadowAtlasResources::compare_sampler)
            .unwrap_or(&self.shadow_compare_sampler);
        let shadow_atlas_slot_buffer = shadow_atlas_resources
            .map(ShadowAtlasResources::slot_buffer)
            .unwrap_or(&self.shadow_atlas_fallback_slot_buffer);
        let shadow_atlas_globals_buffer = shadow_atlas_resources
            .map(ShadowAtlasResources::globals_buffer)
            .unwrap_or(&self.shadow_atlas_fallback_globals_buffer);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-deferred-lighting-bind-group"),
            layout: &self.lighting_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(gbuffer_albedo_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(background_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(gbuffer_material_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(scene_depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: SHADOW_ATLAS_BINDING,
                    resource: wgpu::BindingResource::TextureView(shadow_atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: SHADOW_ATLAS_SAMPLER_BINDING,
                    resource: wgpu::BindingResource::Sampler(shadow_atlas_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: SHADOW_ATLAS_SLOT_BUFFER_BINDING,
                    resource: shadow_atlas_slot_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: SHADOW_GLOBALS_BINDING,
                    resource: shadow_atlas_globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 20,
                    resource: light_grid_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 21,
                    resource: light_zbins_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 22,
                    resource: light_tile_masks_buffer.as_entire_binding(),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("DeferredLightingPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: scene_color_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::BLACK),
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        if !render_region.apply_local_to_render_pass(&mut pass) {
            return;
        }
        pass.set_pipeline(&self.lighting_pipeline);
        pass.set_bind_group(0, scene_bind_group, &[]);
        pass.set_bind_group(1, &bind_group, &[]);
        pass.set_bind_group(3, gpu_scene_bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
