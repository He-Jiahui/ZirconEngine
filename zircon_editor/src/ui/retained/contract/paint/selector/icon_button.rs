#[path = "icon_button/model.rs"]
mod model;
#[path = "icon_button/palette.rs"]
mod palette;
#[path = "icon_button/selection.rs"]
mod selection;
#[path = "icon_button/state.rs"]
mod state;

#[cfg(test)]
#[path = "icon_button/tests/cases.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::{
    WorkbenchIconButtonContext, WorkbenchIconButtonStyle,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_icon_button_style;
