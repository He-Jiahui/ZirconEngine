use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_alert_glyphs::push_close_mark;
use super::super::layout::{WorkbenchToastMetrics, frame_is_within, toast_action_rect};
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
    metrics: WorkbenchToastMetrics,
    opacity: f32,
) {
    let action_rect = toast_action_rect(rect, close, metrics);
    if !frame_is_within(&action_rect, rect) || action_rect.height < metrics.line_height {
        return;
    }
    commands.push(HostPaintCommand::text(
        action_rect,
        Some(clip.clone()),
        order,
        TOAST_ACTION_TEXT.to_string(),
        action_color,
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
    push_close_mark(commands, close, clip, order + 1, close_color, opacity);
}
