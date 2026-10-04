mod model;
mod palette;
mod selection;
mod state;

#[cfg(test)]
#[path = "workbench_popup_row/tests/cases.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::{
    WorkbenchPopupRowState, WorkbenchPopupRowStyle,
};
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::WORKBENCH_POPUP_ROW_DANGER_TEXT;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_popup_row_style;
