use crate::ui::retained_host::host_contract::paint_theme::METRICS;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_chevron_size(
) -> f32 {
    METRICS.font_large
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_chevron_right(
) -> f32 {
    METRICS.button_icon_gap
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_chevron_reserve(
) -> f32 {
    dropdown_chevron_size() + dropdown_chevron_right() + METRICS.gap_s
}
