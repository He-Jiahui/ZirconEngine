#[path = "../data_grid/commands.rs"]
mod commands;
#[path = "../data_grid/content.rs"]
mod content;
#[path = "../data_grid/identity.rs"]
mod identity;
#[path = "../data_grid/metrics.rs"]
mod metrics;
#[path = "data_grid/rows.rs"]
mod rows;
#[path = "../data_grid/surface.rs"]
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_data_grid;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use identity::is_data_grid;
