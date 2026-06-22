use super::super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::super::paint_primitives::draw_text_bars_clipped;
use super::super::super::style::{WELCOME_SUCCESS, WELCOME_WARNING};

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
