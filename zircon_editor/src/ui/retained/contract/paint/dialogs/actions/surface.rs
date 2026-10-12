//! 动作底面的提交边界；按钮必须完整属于根对话框和clip，空间不足时不画部分按钮。

use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::{layout, metrics::dialog_metrics, style::DialogActionPaint};

pub(super) fn push_dialog_action_surface(
    commands: &mut Vec<HostPaintCommand>,
    dialog: &FrameRect,
    frame: FrameRect,
    clip: &FrameRect,
    order: i32,
    paint: DialogActionPaint,
    opacity: f32,
) {
    if !layout::frame_is_within(dialog, &frame) || !layout::frame_is_within(clip, &frame) {
        return;
    }

    let metrics = dialog_metrics();
    commands.push(HostPaintCommand::quad(
        frame,
        Some(clip.clone()),
        order,
        Some(paint.surface),
        Some(paint.border),
        metrics.border_width,
        metrics.action_radius,
        opacity,
    ));
}
