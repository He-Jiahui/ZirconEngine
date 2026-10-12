#[path = "action/close.rs"]
mod close;
#[path = "action/entry.rs"]
mod entry;
#[path = "action/line.rs"]
mod line;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_alert_action;
