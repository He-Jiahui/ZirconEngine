use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::{select_workbench_table_row_style, WorkbenchTableRowStyle};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_cell_color(
    node: &TemplatePaneNodeData,
    index: usize,
) -> [u8; 4] {
    table_row_style(node).text_for_cell(index)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_row_background(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    table_row_style(node).background
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_row_border(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    table_row_style(node).border
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_row_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    table_row_style(node).border_width
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_row_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchTableRowStyle {
    select_workbench_table_row_style(node)
}
