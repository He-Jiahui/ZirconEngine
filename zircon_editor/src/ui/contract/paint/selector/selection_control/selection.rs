#[path = "../../../../paint/selector/selection/selection_control_border.rs"]
mod border;
#[path = "../../../../retained/contract/paint/selector/selection/entry.rs"]
mod entry;
#[path = "../../../../retained/contract/paint/selector/selection/surface.rs"]
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::select_workbench_selection_control_style;
