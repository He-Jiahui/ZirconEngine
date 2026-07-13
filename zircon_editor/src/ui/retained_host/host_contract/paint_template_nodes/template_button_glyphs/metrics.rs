use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

const MIN_BUTTON_ICON_SIZE: f32 = 1.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_icon_size() -> f32 {
    button_icon_size_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_icon_size_from_host(
    metrics: HostControlMetrics,
) -> f32 {
    // Icon16 is the compact-row content slot, not a derivative of title text.
    (metrics.row_height - metrics.gap_l).max(MIN_BUTTON_ICON_SIZE)
}
