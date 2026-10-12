//! toast 尾部动作与关闭标记的 fallback 呈现；这里仅构造文字/图标命令，不绑定操作。
//! 上层只在空间足够时调用，交互层必须另行提供与可见反馈一致的动作语义。

use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_alert_glyphs::push_close_mark;
use super::super::layout::{frame_is_within, toast_action_rect, WorkbenchToastMetrics};
use crate::ui::retained_host::host_contract::data::FrameRect;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

// TODO: [CR-EDITOR-PAINT-OVERLAY-0007] 尾部文字固定为UNDO，当前函数没有消费投影的actions或本地化文案；
// 确认哪条真实交互链保证它可撤销，否则应从动作声明提供内容并同步关闭/点击语义。
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
