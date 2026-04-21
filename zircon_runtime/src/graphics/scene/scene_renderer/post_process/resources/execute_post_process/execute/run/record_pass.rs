use super::super::super::super::super::scene_post_process_resources::ScenePostProcessResources;

pub(super) fn record_pass(
    resources: &ScenePostProcessResources,
    encoder: &mut wgpu::CommandEncoder,
    final_color_view: &wgpu::TextureView,
    global_illumination_view: &wgpu::TextureView,
    bind_group: &wgpu::BindGroup,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("PostProcessPass"),
        color_attachments: &[
            Some(wgpu::RenderPassColorAttachment {
                view: final_color_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: global_illumination_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            }),
        ],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    });
    pass.set_pipeline(&resources.post_process_pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.draw(0..3, 0..1);
}
