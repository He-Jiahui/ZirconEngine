//! 把占位标签事实交给集中样式选择器，确保文字颜色与表面状态来自同一节点。

use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::{select_workbench_dropdown_style, WorkbenchDropdownStyle};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_style(
    node: &TemplatePaneNodeData,
    label_is_placeholder: bool,
) -> WorkbenchDropdownStyle {
    select_workbench_dropdown_style(node, label_is_placeholder)
}
