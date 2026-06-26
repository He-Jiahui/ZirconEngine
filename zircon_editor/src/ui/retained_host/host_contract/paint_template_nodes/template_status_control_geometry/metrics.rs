use crate::ui::retained_host::host_contract::paint_theme::METRICS;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_font_size() -> f32 {
    METRICS.font_body
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_chip_radius() -> f32
{
    METRICS.radius_control
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_icon_button_radius(
) -> f32 {
    METRICS.radius_control
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_line_height() -> f32
{
    METRICS.line_height(status_font_size())
}
