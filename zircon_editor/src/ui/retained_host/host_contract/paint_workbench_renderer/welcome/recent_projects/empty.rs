use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_primitives::draw_text_bars_clipped;
use super::super::style::WELCOME_MUTED_TEXT;

pub(super) fn draw_recent_projects_empty_state(
    frame: &mut HostRgbaFrame,
    list: &FrameRect,
    clip: &FrameRect,
) {
    draw_text_bars_clipped(
        frame,
        list.x + 14.0,
        list.y + 16.0,
        "No recent projects",
        Some(clip),
        WELCOME_MUTED_TEXT,
    );
    draw_text_bars_clipped(
        frame,
        list.x + 14.0,
        list.y + 40.0,
        "Create a new project to start",
        Some(clip),
        WELCOME_MUTED_TEXT,
    );
}
