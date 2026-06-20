use super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract) fn diagnostic_visible_frame(
    frame: &FrameRect,
) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.5
        && frame.height > 0.5
}
