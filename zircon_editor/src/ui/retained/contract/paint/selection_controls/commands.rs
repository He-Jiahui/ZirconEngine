//! 选择控件归属与可见性入口；部分被 clip 裁切时整项被接管但不绘制，需核对这一策略。

// TODO: [CR-EDITOR-PAINT-FORMS-0003] 上游模板节点接受部分相交，此处要求完整落入 clip；确认边缘可见的选择控件是否应整体消失。
use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_selection_control_geometry::{
    frame_is_within, has_paintable_selection_control_extent,
};
use super::checkbox::push_checkbox;
use super::identity::{selection_control_kind, SelectionControlKind};
use super::radio::push_radio;
use super::toggle::push_toggle;

/// 返回值表示选控家族归属；当前完整 clip 容纳是出命令的前提，退化/被裁切时仍返回 true 阻止后续家族回退。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_selection_control_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !has_paintable_selection_control_extent(rect) || !frame_is_within(rect, clip) {
        return selection_control_kind(node).is_some();
    }
    match selection_control_kind(node) {
        Some(SelectionControlKind::Checkbox) => {
            push_checkbox(commands, node, rect, clip, order, opacity);
            true
        }
        Some(SelectionControlKind::Radio) => {
            push_radio(commands, node, rect, clip, order, opacity);
            true
        }
        Some(SelectionControlKind::Toggle) => {
            push_toggle(commands, node, rect, clip, order, opacity);
            true
        }
        None => false,
    }
}
