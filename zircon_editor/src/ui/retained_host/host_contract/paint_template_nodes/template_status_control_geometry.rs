mod chips;
mod common;
mod icons;
mod metrics;
mod signals;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use chips::{
    status_chip_chevron_rect, status_chip_text_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use common::status_control_offset_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use icons::status_icon_button_glyph_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    status_line_height, STATUS_CHIP_RADIUS, STATUS_FONT_SIZE, STATUS_ICON_BUTTON_RADIUS,
};
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use signals::status_signal_text_gap;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use signals::{
    status_signal_icon_paint_rect, status_signal_icon_rect, status_signal_text_rect,
};
