use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::frame_geometry::union_optional_frames as union_host_frames;

pub(in super::super) fn union_optional_frames(
    left: Option<FrameRect>,
    right: Option<FrameRect>,
) -> Option<FrameRect> {
    union_host_frames(left, right)
}
