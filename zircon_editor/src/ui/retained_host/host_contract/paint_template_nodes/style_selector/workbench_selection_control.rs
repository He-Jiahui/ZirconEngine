mod colors;
mod model;
mod palette;
mod selection;
mod state;
#[cfg(test)]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::{
    WorkbenchSelectionControlKind, WorkbenchSelectionControlStyle,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::{
    WORKBENCH_CHECKBOX_CHECKED_FILL, WORKBENCH_RADIO_CHECKED_BORDER, WORKBENCH_RADIO_CHECKED_FILL,
    WORKBENCH_SELECTION_LABEL_MUTED, WORKBENCH_SELECTION_MARK_IDLE_BORDER,
    WORKBENCH_SELECTION_MARK_IDLE_FILL,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_selection_control_style;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use state::is_workbench_selection_state_hot;
