//! Tooltip 绘制前解析共享状态，再应用允许的声明色；调用端已识别提示组件并验证可绘制尺寸。

use super::super::resolved_state_for_node;
use super::colors::apply_declared_tooltip_colors;
use super::model::WorkbenchTooltipStyle;
use super::state::tooltip_state_style;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterFamily;

/// 在提示组件身份和尺寸判定后选择视觉配方；不执行几何或文本布局。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_tooltip_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchTooltipStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Tooltip);
    let mut style = tooltip_state_style(state);

    apply_declared_tooltip_colors(node, &mut style);

    style
}
