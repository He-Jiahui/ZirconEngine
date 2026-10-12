#[path = "alert/colors.rs"]
mod colors;
#[path = "alert/model.rs"]
mod model;
#[path = "alert/palette.rs"]
mod palette;
#[path = "alert/selection.rs"]
mod selection;
#[path = "alert/state.rs"]
mod state;

#[cfg(test)]
#[path = "alert/tests/cases.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::{
    WorkbenchAlertStyle, WorkbenchAlertTone,
};
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::{
    WORKBENCH_ALERT_INFO_SURFACE, WORKBENCH_ALERT_WARNING_SURFACE,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_alert_style;
