use crate::graphics::backend::OffscreenTarget;

/// 按输出和内部渲染尺寸维护离屏目标；返回值供调用方失效依赖尺寸的历史资源。
pub(crate) fn ensure_offscreen_target(
    device: &wgpu::Device,
    target: &mut Option<OffscreenTarget>,
    size: crate::core::math::UVec2,
    render_size: crate::core::math::UVec2,
) -> bool {
    if target
        .as_ref()
        .is_none_or(|offscreen| offscreen.size != size || offscreen.render_size != render_size)
    {
        *target = Some(OffscreenTarget::new_with_render_size(
            device,
            size,
            render_size,
        ));
        return true;
    }

    false
}
