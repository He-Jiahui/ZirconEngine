use super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::intersect;
mod frame;
mod surface;
mod text;

use self::frame::recent_project_row_frames;
use self::surface::{draw_recent_project_row_actions, draw_recent_project_row_surface};
use self::text::draw_recent_project_row_text;
use super::super::super::first_non_empty;

pub(super) fn draw_recent_project_rows(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    list: &FrameRect,
    clip: &FrameRect,
    visible_rows: usize,
) {
    for index in 0..visible_rows {
        let Some(recent) = pane.welcome.recent_projects.row_data(index) else {
            continue;
        };
        let frames = recent_project_row_frames(list, index);
        if intersect(&frames.row, clip).is_none() {
            continue;
        }
        draw_recent_project_row_surface(frame, &frames.row, clip, recent.invalid);
        draw_recent_project_row_text(
            frame,
            &frames.row,
            &frames.text,
            clip,
            recent.display_name.as_str(),
            recent.path.as_str(),
            first_non_empty(&[
                recent.status_label.as_str(),
                recent.last_opened_label.as_str(),
            ]),
            recent.invalid,
        );
        draw_recent_project_row_actions(frame, &frames.open, &frames.remove, clip, recent.invalid);
    }
}
