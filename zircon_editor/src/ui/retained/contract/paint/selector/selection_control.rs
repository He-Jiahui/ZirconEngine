#[path = "../../../../contract/paint/selector/selection_control/colors.rs"]
mod colors;
#[path = "selection_control/model.rs"]
mod model;
#[path = "../../../../contract/paint/selector/selection_control/palette.rs"]
mod palette;
#[path = "../../../../contract/paint/selector/selection_control/selection.rs"]
mod selection;
#[path = "selection_control/state.rs"]
mod state;
#[cfg(test)]
#[path = "../../../../contract/paint/selector/tests/selection_control_cases.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::{
    WorkbenchSelectionControlKind, WorkbenchSelectionControlStyle,
};
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::{
    WORKBENCH_CHECKBOX_CHECKED_FILL, WORKBENCH_RADIO_CHECKED_BORDER, WORKBENCH_RADIO_CHECKED_FILL,
    WORKBENCH_SELECTION_LABEL_MUTED, WORKBENCH_SELECTION_MARK_IDLE_BORDER,
    WORKBENCH_SELECTION_MARK_IDLE_FILL,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_selection_control_style;
