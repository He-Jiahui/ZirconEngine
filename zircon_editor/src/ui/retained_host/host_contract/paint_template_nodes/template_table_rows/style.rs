use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::{select_workbench_table_row_style, WorkbenchTableRowStyle};

pub(super) fn table_cell_color(node: &TemplatePaneNodeData, index: usize) -> [u8; 4] {
    table_row_style(node).text_for_cell(index)
}

pub(super) fn table_row_background(node: &TemplatePaneNodeData) -> [u8; 4] {
    table_row_style(node).background
}

pub(super) fn table_row_border(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    table_row_style(node).border
}

pub(super) fn table_row_border_width(node: &TemplatePaneNodeData) -> f32 {
    table_row_style(node).border_width
}

pub(super) fn table_row_style(node: &TemplatePaneNodeData) -> WorkbenchTableRowStyle {
    select_workbench_table_row_style(node)
}
