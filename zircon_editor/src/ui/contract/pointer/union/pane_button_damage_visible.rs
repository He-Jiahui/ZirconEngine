use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::frame_geometry::{union_frame, visible_frame};

pub(in super::super) fn union_visible_frame(
    current: Option<FrameRect>,
    frame: FrameRect,
) -> Option<FrameRect> {
    if !visible_frame(&frame) {
        return current;
    }
    Some(match current {
        Some(current) => union_frame(&current, &frame),
        None => frame,
    })
}
