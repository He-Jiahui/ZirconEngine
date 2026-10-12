//! toast 消息消费宿主 label fallback，且与尾部反馈共享文字预算。
//! 当前提交单行省略策略；动作存在与否须和 entry 的几何判定一致，避免正文进入尾部区域。

use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_node_labels::template_node_label;
use super::super::layout::{frame_is_within, toast_text_rect, WorkbenchToastMetrics};
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) fn push_toast_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    icon: Option<&FrameRect>,
    close: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    has_action: bool,
    metrics: WorkbenchToastMetrics,
    opacity: f32,
) {
    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }

    let Some(text_rect) = toast_text_rect(rect, icon, close, has_action, metrics) else {
        return;
    };
    if !frame_is_within(&text_rect, rect) || text_rect.height < metrics.line_height {
        return;
    }

    // TODO: [CR-EDITOR-PAINT-OVERLAY-0008] WorkbenchToast 的资源说明为240px场景预留双行消息高度，
    // 这里始终提交单行省略。确认真实 toast 的换行合同，并统一资源布局与 painter 策略。
    commands.push(HostPaintCommand::text(
        text_rect,
        Some(clip.clone()),
        order,
        label,
        color,
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
