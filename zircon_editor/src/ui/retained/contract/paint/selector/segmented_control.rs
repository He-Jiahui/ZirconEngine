#[path = "../../../../contract/paint/selector/segmented_control/control.rs"]
mod control;
#[path = "../../../../contract/paint/selector/segmented_control/metrics.rs"]
mod metrics;
#[path = "segmented_control/model.rs"]
mod model;
#[path = "../../../../contract/paint/selector/segmented_control/palette.rs"]
mod palette;
#[path = "../../../../contract/paint/selector/segmented_control/segments.rs"]
mod segments;
#[path = "../../../../contract/paint/selector/segmented_control/selection.rs"]
mod selection;
#[path = "segmented_control/state.rs"]
mod state;
#[path = "segmented_control/text.rs"]
mod text;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::{
    WorkbenchSegmentedControlKind, WorkbenchSegmentedControlStyle,
};
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::{
    WORKBENCH_SEGMENT_IDLE_BACKGROUND, WORKBENCH_SEGMENT_SELECTED_BACKGROUND,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_segmented_control_style;
