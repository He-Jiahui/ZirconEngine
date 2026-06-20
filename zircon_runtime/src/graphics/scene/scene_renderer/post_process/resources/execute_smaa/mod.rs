use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::scene::scene_renderer::post_process::resources::render_region::{
    apply_physical_render_region_to_pass, create_local_terminal_region_params_buffer,
    create_physical_terminal_region_params_buffer,
};
use crate::graphics::scene::scene_renderer::post_process::SMAA_STAGE_FORMAT;
use crate::graphics::types::ViewportRenderRegion;
use crate::render_graph::RenderGraphAttachmentOps;

use super::super::scene_post_process_resources::ScenePostProcessResources;

impl ScenePostProcessResources {
    pub(in crate::graphics::scene::scene_renderer) fn execute_smaa(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        viewport_size: UVec2,
        terminal_input_view: &wgpu::TextureView,
        final_color_view: &wgpu::TextureView,
        attachment_ops: RenderGraphAttachmentOps,
        render_region: ViewportRenderRegion,
    ) {
        let extent = wgpu::Extent3d {
            width: viewport_size.x.max(1),
            height: viewport_size.y.max(1),
            depth_or_array_layers: 1,
        };
        let edge_texture = create_smaa_stage_texture(device, extent, "zircon-smaa-edges");
        let edge_view = edge_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let blend_texture = create_smaa_stage_texture(device, extent, "zircon-smaa-blend");
        let blend_view = blend_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let edge_bind_group = self.smaa_bind_group(
            device,
            "zircon-smaa-edge-bind-group",
            terminal_input_view,
            terminal_input_view,
            ViewportRenderRegion::default(),
            TerminalRegionSpace::Local,
        );
        self.record_smaa_stage(
            encoder,
            "SmaaEdgePass",
            &self.smaa_edge_pipeline,
            &edge_bind_group,
            &edge_view,
            RenderGraphAttachmentOps::clear_store(),
            None,
        );

        let blend_bind_group = self.smaa_bind_group(
            device,
            "zircon-smaa-blend-bind-group",
            terminal_input_view,
            &edge_view,
            ViewportRenderRegion::default(),
            TerminalRegionSpace::Local,
        );
        self.record_smaa_stage(
            encoder,
            "SmaaBlendPass",
            &self.smaa_blend_pipeline,
            &blend_bind_group,
            &blend_view,
            RenderGraphAttachmentOps::clear_store(),
            None,
        );

        let resolve_bind_group = self.smaa_bind_group(
            device,
            "zircon-smaa-resolve-bind-group",
            terminal_input_view,
            &blend_view,
            render_region,
            TerminalRegionSpace::Physical,
        );
        self.record_smaa_stage(
            encoder,
            "SmaaResolvePass",
            &self.smaa_resolve_pipeline,
            &resolve_bind_group,
            final_color_view,
            attachment_ops,
            Some((render_region, TerminalRegionSpace::Physical)),
        );
    }

    fn smaa_bind_group(
        &self,
        device: &wgpu::Device,
        label: &'static str,
        terminal_input_view: &wgpu::TextureView,
        stage_input_view: &wgpu::TextureView,
        render_region: ViewportRenderRegion,
        region_space: TerminalRegionSpace,
    ) -> wgpu::BindGroup {
        let terminal_region_params_buffer = match region_space {
            TerminalRegionSpace::Local => create_local_terminal_region_params_buffer(
                device,
                "zircon-smaa-terminal-region-params",
                render_region,
            ),
            TerminalRegionSpace::Physical => create_physical_terminal_region_params_buffer(
                device,
                "zircon-smaa-terminal-region-params",
                render_region,
            ),
        };
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: &self.smaa_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(terminal_input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(stage_input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: terminal_region_params_buffer.as_entire_binding(),
                },
            ],
        })
    }

    fn record_smaa_stage(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        label: &'static str,
        pipeline: &wgpu::RenderPipeline,
        bind_group: &wgpu::BindGroup,
        output_view: &wgpu::TextureView,
        attachment_ops: RenderGraphAttachmentOps,
        render_region: Option<(ViewportRenderRegion, TerminalRegionSpace)>,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::BLACK),
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        if let Some((render_region, region_space)) = render_region {
            let region_applied = match region_space {
                TerminalRegionSpace::Local => render_region.apply_local_to_render_pass(&mut pass),
                TerminalRegionSpace::Physical => {
                    apply_physical_render_region_to_pass(&mut pass, render_region)
                }
            };
            if !region_applied {
                return;
            }
        }
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

#[derive(Clone, Copy)]
enum TerminalRegionSpace {
    Local,
    Physical,
}

fn create_smaa_stage_texture(
    device: &wgpu::Device,
    extent: wgpu::Extent3d,
    label: &'static str,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: SMAA_STAGE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

#[cfg(test)]
mod tests {
    use super::SMAA_STAGE_FORMAT;

    #[test]
    fn smaa_stage_textures_store_edge_and_blend_weights() {
        assert_eq!(SMAA_STAGE_FORMAT, wgpu::TextureFormat::Rgba8Unorm);
    }
}
