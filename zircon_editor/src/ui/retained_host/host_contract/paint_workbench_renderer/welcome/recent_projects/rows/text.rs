use super::super::super::super::super::data::FrameRect;
use super::super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::super::paint_primitives::draw_text_bars_clipped;
use super::super::super::style::{WELCOME_MUTED_TEXT, WELCOME_TEXT, WELCOME_WARNING};

pub(super) fn draw_recent_project_row_text(
    frame: &mut HostRgbaFrame,
    row: &FrameRect,
    clip: &FrameRect,
    display_name: &str,
    path: &str,
    status: &str,
    invalid: bool,
) {
    draw_text_bars_clipped(
        frame,
        row.x + 12.0,
        row.y + 8.0,
        display_name,
        Some(clip),
        WELCOME_TEXT,
    );
    draw_text_bars_clipped(
        frame,
        row.x + 12.0,
        row.y + 28.0,
        path,
        Some(clip),
        WELCOME_MUTED_TEXT,
    );
    if status.is_empty() {
        return;
    }
    draw_text_bars_clipped(
        frame,
        row.x + row.width - 108.0_f32.min(row.width * 0.38),
        row.y + 8.0,
        status,
        Some(clip),
        if invalid {
            WELCOME_WARNING
        } else {
            WELCOME_MUTED_TEXT
        },
    );
}
