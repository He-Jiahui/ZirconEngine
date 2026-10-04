//! 提示反馈的接管入口；false 表示交给其他 painter，true 表示该节点由本模块负责。
//! 当前合同要求整个反馈面完整位于 clip 内；部分进入裁剪区也不会提交局部提示。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::identity::{workbench_alert_kind, WorkbenchAlertKind};
use super::inline::push_inline_alert;
use super::layout::{frame_is_within, has_paintable_alert_extent, paint_rect};
use super::toast::push_toast;

/// 调用方传入已投影的节点矩形与允许绘制区域；返回 true 时应停止 fallback。
/// 退化尺寸或不完整包含于 clip 的提示仍被接管，避免通用节点重新补画提示内容。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_alert_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    let Some(kind) = workbench_alert_kind(node) else {
        return false;
    };
    if !has_paintable_alert_extent(rect) {
        return true;
    }
    let rect = paint_rect(rect);
    if !frame_is_within(&rect, clip) {
        return true;
    }

    match kind {
        WorkbenchAlertKind::Inline(tone) => {
            push_inline_alert(commands, node, &rect, clip, order, tone, opacity);
        }
        WorkbenchAlertKind::Toast => {
            push_toast(commands, node, &rect, clip, order, opacity);
        }
    }
    true
}
