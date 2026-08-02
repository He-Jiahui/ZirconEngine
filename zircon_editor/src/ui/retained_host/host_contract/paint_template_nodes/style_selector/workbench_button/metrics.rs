use crate::ui::retained_host::host_contract::paint_theme::{
    HostControlMetrics, current_host_metrics,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_button_border_width()
-> f32 {
    workbench_button_border_width_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_button_border_width_from_host(
    metrics: HostControlMetrics,
) -> f32 {
    metrics.border_width
}
