mod commands;
#[cfg(test)]
#[path = "signals/tests/test_accessors.rs"]
mod test_accessors;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_status_signal_item;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use test_accessors::{
    status_signal_icon_fill, status_signal_text_color,
};
