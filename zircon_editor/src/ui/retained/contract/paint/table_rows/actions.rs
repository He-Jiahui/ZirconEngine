#[path = "actions/entry.rs"]
mod entry;
#[path = "actions/geometry.rs"]
mod geometry;
#[path = "actions/metrics.rs"]
mod metrics;
#[path = "actions/palette.rs"]
mod palette;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes::template_table_rows) use metrics::table_action_column_width;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_table_action;
