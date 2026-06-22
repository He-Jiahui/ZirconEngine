use super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_primitives::draw_text_bars_clipped;
use super::super::layout::welcome_node_frame;
use super::super::style::{WELCOME_MUTED_TEXT, WELCOME_TEXT};

pub(super) fn recent_projects_header_frame(
    pane: &PaneData,
    body: &FrameRect,
    recent_panel: &FrameRect,
) -> FrameRect {
    welcome_node_frame(pane, body, "WelcomeRecentHeaderPanel").unwrap_or_else(|| FrameRect {
        x: recent_panel.x,
        y: recent_panel.y + 26.0,
        width: recent_panel.width,
        height: 54.0,
    })
}

pub(super) fn draw_recent_projects_header(
    frame: &mut HostRgbaFrame,
    header: &FrameRect,
    clip: &FrameRect,
) {
    draw_text_bars_clipped(
        frame,
        header.x + 18.0,
        header.y + 6.0,
        "Recent Projects",
        Some(clip),
        WELCOME_TEXT,
    );
    draw_text_bars_clipped(
        frame,
        header.x + 18.0,
        header.y + 30.0,
        "Pinned startup workspace",
        Some(clip),
        WELCOME_MUTED_TEXT,
    );
}
