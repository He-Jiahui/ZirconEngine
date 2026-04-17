use crate::backend::OffscreenTarget;

pub(crate) fn ensure_offscreen_target(
    device: &wgpu::Device,
    target: &mut Option<OffscreenTarget>,
    size: zircon_math::UVec2,
) {
    if target
        .as_ref()
        .is_none_or(|offscreen| offscreen.size != size)
    {
        *target = Some(OffscreenTarget::new(device, size));
    }
}
