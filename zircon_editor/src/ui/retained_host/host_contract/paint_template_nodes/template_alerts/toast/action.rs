use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_alert_glyphs::push_close_mark;
use super::super::layout::toast_action_rect;
use crate::ui::retained_host::host_contract::data::FrameRect;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const TOAST_ACTION_TEXT: &str = "UNDO";

pub(super) fn push_toast_action(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    close: &FrameRect,
    clip: &FrameRect,
    order: i32,
    action_color: [u8; 4],
    close_color: [u8; 4],
    font_size: f32,
    line_height: f32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::text(
        toast_action_rect(rect, close),
        Some(clip.clone()),
        order,
        TOAST_ACTION_TEXT.to_string(),
        action_color,
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
    push_close_mark(commands, close, clip, order + 1, close_color, opacity);
}
