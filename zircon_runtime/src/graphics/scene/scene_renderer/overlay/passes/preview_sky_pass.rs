use crate::graphics::scene::scene_renderer::attachment_ops::{
    color_attachment_operations, depth_attachment_operations,
};
use crate::graphics::types::{ViewportRenderFrame, ViewportRenderRegion};
use crate::render_graph::RenderGraphAttachmentOps;

pub(crate) struct PreviewSkyPass;

impl PreviewSkyPass {
    pub(crate) fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        sky_pipeline: &wgpu::RenderPipeline,
        volumetric_layout: &wgpu::BindGroupLayout,
        volumetric_apply: &crate::graphics::scene::scene_renderer::advanced_lighting::froxel::VolumetricApplyFallbackResources,
        frame: &ViewportRenderFrame,
    ) {
        self.record_with_attachment_ops(
            encoder,
            device,
            color_view,
            depth_view,
            scene_bind_group,
            sky_pipeline,
            volumetric_layout,
            volumetric_apply,
            None,
            frame,
            frame.render_region(),
            RenderGraphAttachmentOps::clear_store(),
            RenderGraphAttachmentOps::clear_store(),
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_with_attachment_ops(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        sky_pipeline: &wgpu::RenderPipeline,
        volumetric_layout: &wgpu::BindGroupLayout,
        volumetric_apply: &crate::graphics::scene::scene_renderer::advanced_lighting::froxel::VolumetricApplyFallbackResources,
        integrated_volumetric_view: Option<&wgpu::TextureView>,
        frame: &ViewportRenderFrame,
        render_region: ViewportRenderRegion,
        color_attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) {
        let skybox_enabled = frame.environment().skybox.is_enabled();
        let volumetric_binding = skybox_enabled.then(|| {
            let params_buffer = volumetric_apply.create_params_buffer(
                device,
                frame,
                render_region,
                integrated_volumetric_view.is_some(),
                "zircon-sky-volumetric-params",
            );
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("zircon-sky-volumetric-bind-group"),
                layout: volumetric_layout,
                entries: &volumetric_apply
                    .bind_group_entries(&params_buffer, integrated_volumetric_view),
            });
            (params_buffer, bind_group)
        });
        let clear_color = frame.preview().clear_color;
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("PreviewSkyPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(
                    color_attachment_ops,
                    wgpu::Color {
                        r: clear_color.x as f64,
                        g: clear_color.y as f64,
                        b: clear_color.z as f64,
                        a: clear_color.w as f64,
                    },
                ),
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
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
        if let Some((_params_buffer, bind_group)) = &volumetric_binding {
            pass.set_bind_group(0, scene_bind_group, &[]);
            pass.set_bind_group(1, bind_group, &[]);
            pass.set_pipeline(sky_pipeline);
            pass.draw(0..3, 0..1);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn disabled_sky_skips_volumetric_gpu_objects_before_recording() {
        let source = include_str!("preview_sky_pass.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("preview sky implementation");
        let enabled_guard = implementation
            .find("let skybox_enabled = frame.environment().skybox.is_enabled()")
            .expect("skybox enabled guard");
        let params_buffer = implementation
            .find("volumetric_apply.create_params_buffer")
            .expect("volumetric params buffer creation");

        assert!(enabled_guard < params_buffer);
        assert!(implementation.contains("let volumetric_binding = skybox_enabled.then(||"));
        assert!(implementation.contains("if let Some((_params_buffer, bind_group))"));
    }
}
