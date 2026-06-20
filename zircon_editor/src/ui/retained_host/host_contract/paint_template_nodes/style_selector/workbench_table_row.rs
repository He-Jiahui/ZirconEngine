mod colors;
mod identity;
mod model;
mod palette;
mod selection;
mod state;
mod text;

#[cfg(test)]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::WorkbenchTableRowStyle;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::{
    WORKBENCH_TABLE_HEADER_BG, WORKBENCH_TABLE_HEADER_TEXT, WORKBENCH_TABLE_HOVER_BG,
    WORKBENCH_TABLE_ROW_BG, WORKBENCH_TABLE_SELECTED_BG, WORKBENCH_TABLE_SEPARATOR,
    WORKBENCH_TABLE_TAIL_BG,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_table_row_style;
