#[path = "tree_row/colors.rs"]
mod colors;
#[path = "tree_row/model.rs"]
mod model;
#[path = "tree_row/palette.rs"]
mod palette;
#[path = "tree_row/selection.rs"]
mod selection;
#[path = "tree_row/state.rs"]
mod state;
#[path = "tree_row/surface.rs"]
mod surface;

#[cfg(test)]
#[path = "tree_row/tests/cases.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::WorkbenchTreeRowStyle;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::WORKBENCH_TREE_ROW_TEXT_SELECTED;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_tree_row_style;
