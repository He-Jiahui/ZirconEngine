use super::super::super::super::super::data::FrameRect;
use crate::ui::retained_host::welcome_recent_geometry::welcome_recent_row_geometry;
use zircon_runtime_interface::ui::layout::UiFrame;

pub(super) struct RecentProjectRowFrames {
    pub(super) row: FrameRect,
    pub(super) text: FrameRect,
    pub(super) open: FrameRect,
    pub(super) safe: FrameRect,
    pub(super) recover: FrameRect,
    pub(super) remove: FrameRect,
}

pub(super) fn recent_project_row_frames(list: &FrameRect, index: usize) -> RecentProjectRowFrames {
    let geometry = welcome_recent_row_geometry(
        UiFrame::new(list.x, list.y, list.width, list.height),
        index,
        0.0,
    );
    RecentProjectRowFrames {
        row: frame_rect(geometry.row),
        text: frame_rect(geometry.text),
        open: frame_rect(geometry.open),
        safe: frame_rect(geometry.safe),
        recover: frame_rect(geometry.recover),
        remove: frame_rect(geometry.remove),
    }
}

fn frame_rect(frame: UiFrame) -> FrameRect {
    FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}
