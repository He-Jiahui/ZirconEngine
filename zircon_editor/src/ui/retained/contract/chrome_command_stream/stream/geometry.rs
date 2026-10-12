use crate::ui::retained_host::host_contract::data::FrameRect;

pub(in crate::ui::retained_host::host_contract) fn clamp_surface_size(
    size: (u32, u32),
) -> (u32, u32) {
    (size.0.max(1), size.1.max(1))
}

pub(super) fn visible_frame(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}
