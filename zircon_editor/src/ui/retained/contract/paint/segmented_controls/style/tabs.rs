//! 页签外观按集中选择器的 Tab 家族解析，选中标签使用与强调线相配的文字状态。

use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::style_selector::{
    select_workbench_segmented_control_style, WorkbenchSegmentedControlKind as SegmentedStyleKind,
    WorkbenchSegmentedControlStyle,
};

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_background(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    tab_style(node).background
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let style = tab_style(node);
    if node.checked || node.selected {
        style.selected_text
    } else {
        style.idle_text
    }
}

/// 集中选择器的 Tab 家族入口；由页签绘制器和文字颜色函数共同消费。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchSegmentedControlStyle {
    select_workbench_segmented_control_style(node, SegmentedStyleKind::Tab)
}
