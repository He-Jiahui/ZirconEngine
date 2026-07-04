use super::super::template_dropdown_metrics::workbench_dropdown_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_chevron_size(
) -> f32 {
    workbench_dropdown_metrics().chevron_size
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_chevron_right(
) -> f32 {
    workbench_dropdown_metrics().chevron_right
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_chevron_reserve(
) -> f32 {
    workbench_dropdown_metrics().chevron_reserve
}
