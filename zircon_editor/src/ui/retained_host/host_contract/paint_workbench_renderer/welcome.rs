use super::super::data::{FrameRect, PaneData};
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_primitives::{draw_border_clipped, draw_rect_clipped};
use super::SEPARATOR;

mod layout;
mod main_column;
mod recent_projects;
mod style;

use layout::{WELCOME_COLUMN_INSET, inset_frame, welcome_node_frame};
use main_column::draw_welcome_main_column;
use recent_projects::draw_welcome_recent_projects;
use style::{WELCOME_BACKGROUND, WELCOME_SURFACE};

pub(in crate::ui::retained_host::host_contract) fn draw_welcome_native_content(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
) -> bool {
    if pane.welcome.nodes.row_count() == 0 && pane.welcome.title.is_empty() {
        return false;
    }

    draw_rect_clipped(frame, body.clone(), Some(clip), WELCOME_BACKGROUND);

    let outer = welcome_node_frame(pane, body, "WelcomeOuterPanel")
        .unwrap_or_else(|| inset_frame(body, WELCOME_COLUMN_INSET, WELCOME_COLUMN_INSET));
    let recent_panel =
        welcome_node_frame(pane, body, "WelcomeRecentPanel").unwrap_or_else(|| FrameRect {
            x: outer.x,
            y: outer.y,
            width: 320.0_f32.min(outer.width * 0.34).max(220.0),
            height: outer.height,
        });
    let main_panel =
        welcome_node_frame(pane, body, "WelcomeMainPanel").unwrap_or_else(|| FrameRect {
            x: recent_panel.x + recent_panel.width,
            y: outer.y,
            width: (outer.width - recent_panel.width).max(0.0),
            height: outer.height,
        });

    draw_welcome_panel(frame, &recent_panel, clip, WELCOME_SURFACE);
    draw_welcome_panel(frame, &main_panel, clip, WELCOME_BACKGROUND);
    draw_welcome_recent_projects(frame, pane, body, &recent_panel, clip);
    draw_welcome_main_column(frame, pane, body, &main_panel, clip);
    true
}

fn draw_welcome_panel(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    clip: &FrameRect,
    color: [u8; 4],
) {
    draw_rect_clipped(frame, rect.clone(), Some(clip), color);
    draw_border_clipped(frame, rect.clone(), Some(clip), SEPARATOR);
}
