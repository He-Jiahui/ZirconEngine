use crate::ui::retained_host::host_contract::paint_theme::METRICS;

const DROPDOWN_CHEVRON_SIZE: f32 = 16.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_chevron_size(
) -> f32 {
    DROPDOWN_CHEVRON_SIZE
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_chevron_right(
) -> f32 {
    METRICS.button_icon_gap
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_chevron_reserve(
) -> f32 {
    dropdown_chevron_size() + dropdown_chevron_right() + METRICS.gap_s
}
