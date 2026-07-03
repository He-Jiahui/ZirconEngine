mod chips;
mod helpers;
mod icons;
mod model;
mod palette;
mod signals;
#[cfg(test)]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use chips::select_workbench_status_chip_style;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use icons::select_workbench_status_icon_button_style;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::{
    WorkbenchStatusSignalKind, WorkbenchStatusSignalStyle,
};
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::WORKBENCH_STATUS_NO_ERRORS_FILL;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use signals::select_workbench_status_signal_style;
