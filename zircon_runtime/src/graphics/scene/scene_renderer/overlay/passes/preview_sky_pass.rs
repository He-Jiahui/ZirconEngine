use crate::core::framework::render::FallbackSkyboxKind;

use crate::graphics::types::ViewportRenderFrame;

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
        let clear_color = frame.scene.preview.clear_color;
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("PreviewSkyPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: clear_color.x as f64,
                        g: clear_color.y as f64,
                        b: clear_color.z as f64,
                        a: clear_color.w as f64,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        if frame.scene.preview.skybox_enabled
            && matches!(
                frame.scene.preview.fallback_skybox,
                FallbackSkyboxKind::ProceduralGradient
            )
        {
            pass.set_bind_group(0, scene_bind_group, &[]);
            pass.set_pipeline(sky_pipeline);
            pass.draw(0..3, 0..1);
        }
    }
}
