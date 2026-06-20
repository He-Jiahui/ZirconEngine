use super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_primitives::{
    draw_border_clipped, draw_rect_clipped, draw_text_bars_clipped,
};
use super::super::super::{first_non_empty, SEPARATOR};
use super::super::style::{
    WELCOME_MUTED_TEXT, WELCOME_SUCCESS, WELCOME_SURFACE, WELCOME_TEXT, WELCOME_WARNING,
};

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

pub(in crate::ui::retained_host::host_contract) fn draw_welcome_field(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    label: &str,
    value: &str,
    clip: &FrameRect,
) {
    draw_rect_clipped(frame, rect.clone(), Some(clip), WELCOME_SURFACE);
    draw_border_clipped(frame, rect.clone(), Some(clip), SEPARATOR);
    draw_text_bars_clipped(
        frame,
        rect.x + 14.0,
        rect.y + 8.0,
        label,
        Some(clip),
        WELCOME_MUTED_TEXT,
    );
    draw_text_bars_clipped(
        frame,
        rect.x + 14.0,
        rect.y + 30.0,
        value,
        Some(clip),
        WELCOME_TEXT,
    );
}

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

pub(in crate::ui::retained_host::host_contract) fn draw_welcome_validation(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    validation: &FrameRect,
    clip: &FrameRect,
) {
    let message = if !pane.welcome.form.validation_message.trim().is_empty() {
        pane.welcome.form.validation_message.as_str()
    } else if pane.welcome.form.can_create {
        "Project settings are valid"
    } else {
        "Enter a project name and location"
    };
    let color = if pane.welcome.form.can_create {
        WELCOME_SUCCESS
    } else {
        WELCOME_WARNING
    };
    draw_text_bars_clipped(
        frame,
        validation.x,
        validation.y + 8.0,
        message,
        Some(clip),
        color,
    );
}
