use super::super::super::super::data::FrameRect;
use super::super::super::{UiProfileFrame, UiProfilePoint};

pub(in crate::ui::retained_host::host_contract) fn profile_frame_center(
    frame: &UiProfileFrame,
) -> UiProfilePoint {
    UiProfilePoint {
        x: frame.x + frame.width * 0.5,
        y: frame.y + frame.height * 0.5,
    }
}

pub(in crate::ui::retained_host::host_contract) fn frame_rect_center_point(
    frame: &FrameRect,
) -> UiProfilePoint {
    UiProfilePoint {
        x: frame.x + frame.width * 0.5,
        y: frame.y + frame.height * 0.5,
    }
}
