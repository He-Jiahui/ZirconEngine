#[path = "dropdown/brightness.rs"]
mod brightness;
#[path = "dropdown/colors.rs"]
mod colors;
#[path = "dropdown/model.rs"]
mod model;
#[path = "dropdown/palette.rs"]
mod palette;
#[path = "dropdown/selection.rs"]
mod selection;
#[path = "dropdown/state.rs"]
mod state;

#[cfg(test)]
#[path = "dropdown/tests/cases.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::WorkbenchDropdownStyle;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::workbench_dropdown_palette;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_dropdown_style;
