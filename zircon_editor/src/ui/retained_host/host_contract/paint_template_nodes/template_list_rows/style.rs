use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::{select_workbench_list_row_style, WorkbenchListRowStyle};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_background(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    list_row_style(node).background
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_border(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    list_row_style(node).border
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    list_row_style(node).border_width
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    list_row_style(node).text
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_adornment_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    list_row_style(node).adornment
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchListRowStyle {
    select_workbench_list_row_style(node)
}
