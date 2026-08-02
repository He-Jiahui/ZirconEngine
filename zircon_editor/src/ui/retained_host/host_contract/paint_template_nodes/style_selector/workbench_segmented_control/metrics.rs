use crate::ui::retained_host::host_contract::paint_theme::{
    HostControlMetrics, current_host_metrics,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchSegmentedSelectorMetrics
{
    pub border_width: f32,
    pub selected_underline_height: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_segmented_selector_metrics()
-> WorkbenchSegmentedSelectorMetrics {
    workbench_segmented_selector_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_segmented_selector_metrics_from_host(
    metrics: HostControlMetrics,
) -> WorkbenchSegmentedSelectorMetrics {
    WorkbenchSegmentedSelectorMetrics {
        border_width: metrics.border_width,
        selected_underline_height: metrics.tab_underline_height,
    }
}
