mod colors;
mod model;
mod palette;
mod selection;
mod state;

#[cfg(test)]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::{
    WorkbenchAlertStyle, WorkbenchAlertTone,
};
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::{
    WORKBENCH_ALERT_INFO_SURFACE, WORKBENCH_ALERT_WARNING_SURFACE,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_alert_style;
