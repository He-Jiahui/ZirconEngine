use super::super::super::super::super::data::FrameRect;
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::super::super::template_alert_glyphs::push_close_mark;

pub(super) fn push_alert_close_mark(
    commands: &mut Vec<HostPaintCommand>,
    frame: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_close_mark(commands, frame, clip, order, color, opacity);
}
