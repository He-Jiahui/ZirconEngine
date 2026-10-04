//! 树行选择入口合成共享状态与 selected/checked 持久标记，再给表面和四种内容通道提供统一配方。

use super::super::resolved_state_for_node;
use super::colors::{
    tree_row_action_color, tree_row_icon_color, tree_row_primary_color, tree_row_secondary_color,
};
use super::model::WorkbenchTreeRowStyle;
use super::surface::{tree_row_background, tree_row_border, tree_row_border_width};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterFamily;

/// 为模板树行合成共享状态与持久选择，提供外壳和文字角色。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_tree_row_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchTreeRowStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::TreeRow);
    let marked = node.selected || node.checked;

    WorkbenchTreeRowStyle {
        background: tree_row_background(node, state, marked),
        border: tree_row_border(state, marked),
        border_width: tree_row_border_width(state, marked),
        text: tree_row_primary_color(state, marked),
        icon: tree_row_icon_color(state, marked),
        secondary: tree_row_secondary_color(state, marked),
        action: tree_row_action_color(state, marked),
        state,
    }
}
