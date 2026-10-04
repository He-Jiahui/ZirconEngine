//! Toast 绘制前先选择共享状态配方，再应用允许的节点声明色；不判断消息内容、寿命或动作可用性。

use super::super::resolved_state_for_node;
use super::colors::apply_declared_toast_colors;
use super::model::WorkbenchToastStyle;
use super::state::toast_state_style;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterFamily;

/// 选择 Toast 的状态与声明色配方；调用端决定消息内容和是否绘制操作。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_toast_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchToastStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Toast);
    let mut style = toast_state_style(state);

    apply_declared_toast_colors(node, &mut style);

    style
}
