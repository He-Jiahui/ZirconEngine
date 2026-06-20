use crate::graphics::types::ViewportRenderRegion;

pub(crate) fn begin_line_pass_for_region<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    label: &'static str,
    color_view: &'a wgpu::TextureView,
    depth_view: &'a wgpu::TextureView,
    render_region: ViewportRenderRegion,
) -> Option<wgpu::RenderPass<'a>> {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: color_view,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    });
    render_region
        .apply_physical_to_render_pass(&mut pass)
        .then_some(pass)
}
