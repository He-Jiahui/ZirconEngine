#[path = "tooltip/colors.rs"]
mod colors;
#[path = "tooltip/model.rs"]
mod model;
#[path = "tooltip/palette.rs"]
mod palette;
#[path = "tooltip/selection.rs"]
mod selection;
#[path = "tooltip/state.rs"]
mod state;

#[cfg(test)]
#[path = "tooltip/tests/cases.rs"]
mod tests;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::{
    WORKBENCH_TOOLTIP_BORDER, WORKBENCH_TOOLTIP_SURFACE,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_tooltip_style;
