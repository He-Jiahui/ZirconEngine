#[path = "../pickers/commands.rs"]
mod commands;
#[path = "pickers/field.rs"]
mod field;
#[path = "../pickers/geometry.rs"]
mod geometry;
#[path = "../pickers/identity.rs"]
mod identity;
#[path = "pickers/metrics.rs"]
mod metrics;
#[path = "pickers/popup.rs"]
mod popup;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_date_time_picker;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use identity::is_date_time_picker;
