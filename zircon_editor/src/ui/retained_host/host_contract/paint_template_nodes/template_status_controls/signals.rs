mod commands;
mod metrics;
#[cfg(test)]
mod test_accessors;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_status_signal_item;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::status_signal_mark_width;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use test_accessors::{
    status_signal_icon_fill, status_signal_mark_color, status_signal_text_color,
};
