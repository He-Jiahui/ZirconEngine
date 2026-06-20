mod colors;
mod model;
mod palette;
mod selection;
mod state;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::{
    WORKBENCH_TOAST_ACTION, WORKBENCH_TOAST_BORDER, WORKBENCH_TOAST_SURFACE,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_toast_style;
