use super::super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::super::paint_primitives::draw_text_bars_clipped;
use super::super::super::style::{WELCOME_MUTED_TEXT, WELCOME_TEXT};

pub(in crate::ui::retained_host::host_contract) fn draw_welcome_new_project_header(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    header: &FrameRect,
    clip: &FrameRect,
) {
    draw_text_bars_clipped(
        frame,
        header.x,
        header.y + 2.0,
        "New Project",
        Some(clip),
        WELCOME_TEXT,
    );
    draw_text_bars_clipped(
        frame,
        header.x,
        header.y + 24.0,
        pane.welcome.form.template_label.as_str(),
        Some(clip),
        WELCOME_MUTED_TEXT,
    );
}
