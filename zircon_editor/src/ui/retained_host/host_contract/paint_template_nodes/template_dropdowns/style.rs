use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::{WorkbenchDropdownStyle, select_workbench_dropdown_style};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_style(
    node: &TemplatePaneNodeData,
    label_is_placeholder: bool,
) -> WorkbenchDropdownStyle {
    select_workbench_dropdown_style(node, label_is_placeholder)
}
