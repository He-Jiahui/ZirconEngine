use crate::core::framework::render::FallbackSkyboxKind;

use crate::graphics::scene::scene_renderer::attachment_ops::{
    color_attachment_operations, depth_attachment_operations,
};
use crate::graphics::types::ViewportRenderFrame;
use crate::render_graph::RenderGraphAttachmentOps;

pub(crate) struct PreviewSkyPass;

impl PreviewSkyPass {
    pub(crate) fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        sky_pipeline: &wgpu::RenderPipeline,
        frame: &ViewportRenderFrame,
    ) {
        self.record_with_attachment_ops(
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            sky_pipeline,
            frame,
            RenderGraphAttachmentOps::clear_store(),
            RenderGraphAttachmentOps::clear_store(),
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_with_attachment_ops(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        sky_pipeline: &wgpu::RenderPipeline,
        frame: &ViewportRenderFrame,
        color_attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) {
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
        if !frame
            .render_region()
            .apply_physical_to_render_pass(&mut pass)
        {
            return;
        }
        if frame.preview().skybox_enabled
            && matches!(
                frame.preview().fallback_skybox,
                FallbackSkyboxKind::ProceduralGradient
            )
        {
            pass.set_bind_group(0, scene_bind_group, &[]);
            pass.set_pipeline(sky_pipeline);
            pass.draw(0..3, 0..1);
        }
    }
}
