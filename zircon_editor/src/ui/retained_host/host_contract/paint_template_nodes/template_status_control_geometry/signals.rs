mod constants;
mod icon;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use icon::{
    status_signal_icon_paint_rect, status_signal_icon_rect,
};
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text::status_signal_text_gap;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text::status_signal_text_rect;
