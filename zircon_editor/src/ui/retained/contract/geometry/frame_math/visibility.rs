use super::super::super::super::data::FrameRect;
use super::super::super::UiProfileFrame;

pub(in crate::ui::retained_host::host_contract) fn visible_profile_frame(
    frame: &FrameRect,
) -> Option<UiProfileFrame> {
    is_visible_frame(frame).then(|| frame.into())
}

pub(in crate::ui::retained_host::host_contract) fn is_visible_frame(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

pub(in crate::ui::retained_host::host_contract) fn is_visible_profile_frame(
    frame: &UiProfileFrame,
) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}
