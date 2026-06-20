use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::{select_workbench_dropdown_style, WorkbenchDropdownStyle};
use super::text::dropdown_label_is_placeholder;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchDropdownStyle {
    select_workbench_dropdown_style(node, dropdown_label_is_placeholder(node))
}
