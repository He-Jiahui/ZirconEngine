mod close;
mod marks;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use close::push_close_mark;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use marks::push_alert_mark;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ALERT_ICON_SIZE: f32 =
    18.0;
