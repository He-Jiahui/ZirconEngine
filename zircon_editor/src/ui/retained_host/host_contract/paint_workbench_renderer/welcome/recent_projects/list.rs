use super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_primitives::{draw_border_clipped, draw_rect_clipped};
use super::super::super::SEPARATOR;
use super::super::layout::welcome_node_frame;
use super::super::style::WELCOME_SURFACE_INSET;

pub(super) fn recent_projects_list_frame(
    pane: &PaneData,
    body: &FrameRect,
    recent_panel: &FrameRect,
    header: &FrameRect,
) -> FrameRect {
    welcome_node_frame(pane, body, "WelcomeRecentListPanel").unwrap_or_else(|| FrameRect {
        x: recent_panel.x + 12.0,
        y: header.y + header.height + 14.0,
        width: (recent_panel.width - 24.0).max(0.0),
        height: (recent_panel.height - header.height - 40.0).max(0.0),
    })
}

pub(super) fn draw_recent_projects_list_surface(
    frame: &mut HostRgbaFrame,
    list: &FrameRect,
    clip: &FrameRect,
) {
    draw_rect_clipped(frame, list.clone(), Some(clip), WELCOME_SURFACE_INSET);
    draw_border_clipped(frame, list.clone(), Some(clip), SEPARATOR);
}
