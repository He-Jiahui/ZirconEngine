/// 为停用的效果写入确定的中性输出，防止后续合成读到上一帧残留。
/// 调用者提供可作颜色附件的目标视图，并负责提交包含此清除的命令编码器。
pub(in crate::graphics::scene::scene_renderer::post_process) fn clear_render_target(
    encoder: &mut wgpu::CommandEncoder,
    label: &'static str,
    view: &wgpu::TextureView,
    color: wgpu::Color,
) {
    let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(color),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    });
}
