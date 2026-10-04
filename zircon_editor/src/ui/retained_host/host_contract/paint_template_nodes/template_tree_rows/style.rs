//! 树行视觉全部由共享状态选择器决定；表面、标题、对象和操作读取同一状态优先级。
//! 局部 wrapper 仅投影字段，不能在 glyph 层重新推断 disabled、pressed 或 selected 优先级。

use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::{select_workbench_tree_row_style, WorkbenchTreeRowStyle};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_background(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    tree_row_style(node).background
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_border(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    tree_row_style(node).border
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    tree_row_style(node).border_width
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    tree_row_style(node).text
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_icon_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    tree_row_style(node).icon
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_secondary_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    tree_row_style(node).secondary
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_action_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    tree_row_style(node).action
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_state(
    node: &TemplatePaneNodeData,
) -> UiPainterResolvedState {
    tree_row_style(node).state
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchTreeRowStyle {
    select_workbench_tree_row_style(node)
}
