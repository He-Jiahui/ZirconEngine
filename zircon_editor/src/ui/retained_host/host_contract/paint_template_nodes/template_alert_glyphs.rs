mod close;
mod marks;
mod segments;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use close::push_close_mark;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use marks::push_alert_mark;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use segments::ALERT_ICON_SIZE;
