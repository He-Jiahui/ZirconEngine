#[path = "table_row/colors.rs"]
mod colors;
#[path = "table_row/identity.rs"]
mod identity;
#[path = "table_row/model.rs"]
mod model;
#[path = "table_row/palette.rs"]
mod palette;
#[path = "table_row/selection.rs"]
mod selection;
#[path = "table_row/state.rs"]
mod state;
#[path = "table_row/text.rs"]
mod text;

#[cfg(test)]
#[path = "table_row/tests/cases.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::WorkbenchTableRowStyle;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::{
    WORKBENCH_TABLE_HEADER_BG, WORKBENCH_TABLE_HEADER_TEXT, WORKBENCH_TABLE_HOVER_BG,
    WORKBENCH_TABLE_SELECTED_BG, WORKBENCH_TABLE_SEPARATOR, WORKBENCH_TABLE_TAIL_BG,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_table_row_style;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use state::is_hot as is_hot_workbench_table_row_state;
