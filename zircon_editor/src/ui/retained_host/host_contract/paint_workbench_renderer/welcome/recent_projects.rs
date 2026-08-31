use super::super::super::data::{FrameRect, PaneData, WelcomePaneLayoutData};
use super::super::super::paint_frame::HostRgbaFrame;
use crate::ui::retained_host::welcome_recent_geometry::welcome_recent_visible_row_count;

mod empty;
mod header;
mod list;
mod rows;

use empty::draw_recent_projects_empty_state;
use header::{draw_recent_projects_header, recent_projects_header_frame};
use list::{draw_recent_projects_list_surface, recent_projects_list_frame};
use rows::draw_recent_project_rows;

pub(in crate::ui::retained_host::host_contract) fn draw_welcome_recent_projects(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    layout: &WelcomePaneLayoutData,
    body: &FrameRect,
    recent_panel: &FrameRect,
    clip: &FrameRect,
) {
    let header = recent_projects_header_frame(layout, body, recent_panel);
    draw_recent_projects_header(frame, &header, clip);

    let list = recent_projects_list_frame(layout, body, recent_panel, &header);
    draw_recent_projects_list_surface(frame, &list, clip);

    let row_count = pane.welcome.recent_projects.row_count();
    if row_count == 0 {
        draw_recent_projects_empty_state(frame, &list, clip);
        return;
    }

    draw_recent_project_rows(
        frame,
        pane,
        &list,
        clip,
        welcome_recent_visible_row_count(list.height, row_count),
    );
}
