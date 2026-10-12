#[path = "toast/colors.rs"]
mod colors;
#[path = "toast/model.rs"]
mod model;
#[path = "toast/palette.rs"]
mod palette;
#[path = "toast/selection.rs"]
mod selection;
#[path = "toast/state.rs"]
mod state;

#[cfg(test)]
#[path = "toast/tests/cases.rs"]
mod tests;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::{
    WORKBENCH_TOAST_ACTION, WORKBENCH_TOAST_BORDER, WORKBENCH_TOAST_SURFACE,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_toast_style;
