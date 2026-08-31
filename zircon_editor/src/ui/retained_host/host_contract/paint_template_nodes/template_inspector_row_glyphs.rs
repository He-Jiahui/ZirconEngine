mod checks;
mod chevrons;
mod cubes;
mod swatches;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use checks::push_inspector_check_tick;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use chevrons::push_inspector_down_chevron;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use cubes::push_inspector_cube_icon;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use swatches::push_inspector_swatch;
