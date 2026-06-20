mod control;
mod model;
mod palette;
mod segments;
mod selection;
mod state;
mod text;

#[cfg(test)]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::{
    WorkbenchSegmentedControlKind, WorkbenchSegmentedControlStyle,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::WORKBENCH_SEGMENT_IDLE_BACKGROUND;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::select_workbench_segmented_control_style;
