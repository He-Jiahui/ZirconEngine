use super::super::super::data::{FrameRect, PaneData, WelcomePaneLayoutData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::is_visible_frame;

mod form;
mod frames;
mod hero;

use form::{draw_welcome_new_project_header, draw_welcome_preview, draw_welcome_validation};
use frames::welcome_main_column_frames;
use hero::{draw_welcome_hero, draw_welcome_status};

pub(in crate::ui::retained_host::host_contract) fn draw_welcome_main_column(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    layout: &WelcomePaneLayoutData,
    body: &FrameRect,
    main_panel: &FrameRect,
    clip: &FrameRect,
) {
    let frames = welcome_main_column_frames(layout, body, main_panel);
    if is_visible_frame(&frames.hero) {
        draw_welcome_hero(frame, pane, &frames.hero, clip);
    }

    if is_visible_frame(&frames.status) {
        draw_welcome_status(frame, pane, &frames.status, clip);
    }

    if is_visible_frame(&frames.header) {
        draw_welcome_new_project_header(frame, pane, &frames.header, clip);
    }

    if is_visible_frame(&frames.preview) {
        draw_welcome_preview(frame, pane, &frames.preview, clip);
    }

    if is_visible_frame(&frames.validation) {
        draw_welcome_validation(frame, pane, &frames.validation, clip);
    }
}
