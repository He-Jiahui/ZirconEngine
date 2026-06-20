use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::metrics::{DIALOG_ACTION_FONT_SIZE, DIALOG_ACTION_LINE_HEIGHT};

pub(super) fn push_dialog_action_text(
    commands: &mut Vec<HostPaintCommand>,
    frame: FrameRect,
    clip: &FrameRect,
    order: i32,
    text: String,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::text(
        frame,
        Some(clip.clone()),
        order,
        text,
        color,
        DIALOG_ACTION_FONT_SIZE,
        DIALOG_ACTION_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
