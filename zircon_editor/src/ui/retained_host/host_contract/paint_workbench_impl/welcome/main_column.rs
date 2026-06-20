use super::super::super::data::{FrameRect, PaneData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::layout::{constrain_welcome_content, welcome_node_frame, WELCOME_CONTENT_MAX_WIDTH};

mod actions;
mod form;
mod hero;

use actions::draw_welcome_actions;
use form::{
    draw_welcome_field, draw_welcome_new_project_header, draw_welcome_preview,
    draw_welcome_validation,
};
use hero::{draw_welcome_hero, draw_welcome_status};

pub(in crate::ui::retained_host::host_contract) fn draw_welcome_main_column(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    main_panel: &FrameRect,
    clip: &FrameRect,
) {
    let content_x = main_panel.x + 28.0;
    let content_width = (main_panel.width - 56.0)
        .max(0.0)
        .min(WELCOME_CONTENT_MAX_WIDTH);
    let hero = constrain_welcome_content(
        welcome_node_frame(pane, body, "WelcomeHeroPanel").unwrap_or_else(|| FrameRect {
            x: content_x,
            y: main_panel.y + 28.0,
            width: content_width,
            height: 84.0,
        }),
        content_x,
        content_width,
    );
    draw_welcome_hero(frame, pane, &hero, clip);

    let status = constrain_welcome_content(
        welcome_node_frame(pane, body, "WelcomeStatusPanel").unwrap_or_else(|| FrameRect {
            x: content_x,
            y: hero.y + hero.height + 12.0,
            width: content_width,
            height: 30.0,
        }),
        content_x,
        content_width,
    );
    draw_welcome_status(frame, pane, &status, clip);

    let header = constrain_welcome_content(
        welcome_node_frame(pane, body, "WelcomeNewProjectHeaderPanel").unwrap_or_else(|| {
            FrameRect {
                x: content_x,
                y: status.y + status.height + 22.0,
                width: content_width,
                height: 46.0,
            }
        }),
        content_x,
        content_width,
    );
    draw_welcome_new_project_header(frame, pane, &header, clip);

    let name = constrain_welcome_content(
        welcome_node_frame(pane, body, "WelcomeProjectNameField").unwrap_or_else(|| FrameRect {
            x: content_x,
            y: header.y + header.height + 16.0,
            width: content_width,
            height: 56.0,
        }),
        content_x,
        content_width,
    );
    draw_welcome_field(
        frame,
        &name,
        "Project name",
        pane.welcome.form.project_name.as_str(),
        clip,
    );

    let location = constrain_welcome_content(
        welcome_node_frame(pane, body, "WelcomeLocationField").unwrap_or_else(|| FrameRect {
            x: content_x,
            y: name.y + name.height + 12.0,
            width: content_width,
            height: 56.0,
        }),
        content_x,
        content_width,
    );
    draw_welcome_field(
        frame,
        &location,
        "Location",
        pane.welcome.form.location.as_str(),
        clip,
    );

    let preview = constrain_welcome_content(
        welcome_node_frame(pane, body, "WelcomePreviewPanel").unwrap_or_else(|| FrameRect {
            x: content_x,
            y: location.y + location.height + 14.0,
            width: content_width,
            height: 72.0,
        }),
        content_x,
        content_width,
    );
    draw_welcome_preview(frame, pane, &preview, clip);

    let validation = constrain_welcome_content(
        welcome_node_frame(pane, body, "WelcomeValidationPanel").unwrap_or_else(|| FrameRect {
            x: content_x,
            y: preview.y + preview.height + 10.0,
            width: content_width,
            height: 36.0,
        }),
        content_x,
        content_width,
    );
    draw_welcome_validation(frame, pane, &validation, clip);

    let actions = constrain_welcome_content(
        welcome_node_frame(pane, body, "WelcomeActionsRow").unwrap_or_else(|| FrameRect {
            x: content_x,
            y: validation.y + validation.height + 12.0,
            width: content_width,
            height: 32.0,
        }),
        content_x,
        content_width,
    );
    draw_welcome_actions(frame, pane, &actions, clip);
}
