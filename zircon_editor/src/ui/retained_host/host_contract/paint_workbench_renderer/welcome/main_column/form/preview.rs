use super::super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::super::paint_primitives::{
    draw_border_clipped, draw_rect_clipped, draw_text_bars_clipped,
};
use super::super::super::super::{first_non_empty, SEPARATOR};
use super::super::super::style::{WELCOME_MUTED_TEXT, WELCOME_SURFACE, WELCOME_TEXT};

pub(in crate::ui::retained_host::host_contract) fn draw_welcome_preview(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    preview: &FrameRect,
    clip: &FrameRect,
) {
    draw_rect_clipped(frame, preview.clone(), Some(clip), WELCOME_SURFACE);
    draw_border_clipped(frame, preview.clone(), Some(clip), SEPARATOR);
    draw_text_bars_clipped(
        frame,
        preview.x + 14.0,
        preview.y + 10.0,
        "Project path",
        Some(clip),
        WELCOME_MUTED_TEXT,
    );
    draw_text_bars_clipped(
        frame,
        preview.x + 14.0,
        preview.y + 36.0,
        first_non_empty(&[
            pane.welcome.form.project_path_preview.as_str(),
            "Project path will appear here",
        ]),
        Some(clip),
        if pane.welcome.form.project_path_preview.is_empty() {
            WELCOME_MUTED_TEXT
        } else {
            WELCOME_TEXT
        },
    );
}
