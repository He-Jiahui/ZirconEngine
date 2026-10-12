//! 节点专用链以固定身份识别状态控件；已认领但尺寸无效时仍返回 true，阻止通用 fallback。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_status_control_geometry::has_paintable_status_control_extent;
use super::chips::push_status_chip;
use super::icons::push_status_icon_button;
use super::identity::{status_control_kind, StatusControlKind};
use super::signals::push_status_signal_item;

/// 分派单个状态栏节点；信号可无背景，chip/图标必须尊重视觉偏移和父裁剪。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_status_control_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    let Some(kind) = status_control_kind(node) else {
        return false;
    };
    if !has_paintable_status_control_extent(rect) {
        return true;
    }
    match kind {
        StatusControlKind::Signal(kind) => {
            push_status_signal_item(commands, node, rect, clip, order, kind, opacity);
            true
        }
        StatusControlKind::Chip => {
            push_status_chip(commands, node, rect, clip, order, opacity);
            true
        }
        StatusControlKind::Icon(kind) => {
            push_status_icon_button(commands, node, rect, clip, order, kind, opacity);
            true
        }
    }
}
