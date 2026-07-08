use super::super::super::super::super::data::FrameRect;
use super::super::super::geometry::local_rect;

pub(super) fn folder_body_rect(rect: &FrameRect) -> FrameRect {
    local_rect(rect, 2.0, 5.0, 10.0, 7.0)
}

pub(super) fn folder_tab_rect(rect: &FrameRect) -> FrameRect {
    local_rect(rect, 3.0, 3.0, 5.0, 3.0)
}

pub(super) fn save_body_rect(rect: &FrameRect) -> FrameRect {
    local_rect(rect, 2.0, 2.0, 10.0, 10.0)
}

pub(super) fn save_top_cutout_rect(rect: &FrameRect) -> FrameRect {
    local_rect(rect, 4.0, 3.0, 5.0, 3.0)
}

pub(super) fn save_bottom_cutout_rect(rect: &FrameRect) -> FrameRect {
    local_rect(rect, 4.0, 9.0, 6.0, 2.0)
}
