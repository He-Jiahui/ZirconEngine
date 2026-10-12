//! 动作文字的独立提交边界；消费labels测量好的按钮内区域，不在这里重新布局按钮或解析动作。

use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::metrics::dialog_metrics;

pub(super) fn push_dialog_action_text(
    commands: &mut Vec<HostPaintCommand>,
    dialog: &FrameRect,
    frame: FrameRect,
    clip: &FrameRect,
    order: i32,
    text: String,
    color: [u8; 4],
    opacity: f32,
) {
    if !super::super::layout::frame_is_within(dialog, &frame)
        || !super::super::layout::frame_is_within(clip, &frame)
    {
        return;
    }

    let metrics = dialog_metrics();
    commands.push(HostPaintCommand::text(
        frame,
        Some(clip.clone()),
        order,
        text,
        color,
        metrics.action_font_size,
        metrics.action_line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
