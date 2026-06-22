use super::super::super::super::super::data::FrameRect;
use super::super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::super::paint_primitives::{draw_border_clipped, draw_rect_clipped};
use super::super::super::super::SEPARATOR;
use super::super::super::style::{WELCOME_SURFACE, WELCOME_WARNING};

pub(super) fn draw_recent_project_row_surface(
    frame: &mut HostRgbaFrame,
    row: &FrameRect,
    clip: &FrameRect,
    invalid: bool,
) {
    draw_rect_clipped(frame, row.clone(), Some(clip), WELCOME_SURFACE);
    draw_border_clipped(
        frame,
        row.clone(),
        Some(clip),
        if invalid { WELCOME_WARNING } else { SEPARATOR },
    );
}
