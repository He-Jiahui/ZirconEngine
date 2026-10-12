//! 分段组与页签共用接管布尔协议；无有效选项的分段组让后续普通按钮或通用回退继续处理。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::identity::{is_segmented_control, is_workbench_tab};
use super::options::segmented_option_count;
use super::segments::push_segmented_control;
use super::tabs::push_workbench_tab;

/// primary 链先尝试分段组再尝试页签；有归属但无可见命令时仍返回 true，无有效选项的组返回 false。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_segmented_control_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if is_segmented_control(node) {
        let option_count = segmented_option_count(node);
        if option_count == 0 {
            return false;
        }
        push_segmented_control(commands, node, rect, clip, order, opacity, option_count);
        return true;
    }

    if is_workbench_tab(node) {
        push_workbench_tab(commands, node, rect, clip, order, opacity);
        return true;
    }

    false
}
