mod brightness;
mod colors;
mod model;
mod palette;
mod selection;
mod state;

#[cfg(test)]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::WorkbenchDropdownStyle;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::{
    WORKBENCH_DROPDOWN_BORDER, WORKBENCH_DROPDOWN_FOCUS_BORDER, WORKBENCH_DROPDOWN_PLACEHOLDER,
    WORKBENCH_DROPDOWN_SURFACE,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_dropdown_style;
