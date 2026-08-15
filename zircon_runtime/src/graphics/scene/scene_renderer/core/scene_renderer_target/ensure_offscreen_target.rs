use crate::graphics::backend::OffscreenTarget;

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
