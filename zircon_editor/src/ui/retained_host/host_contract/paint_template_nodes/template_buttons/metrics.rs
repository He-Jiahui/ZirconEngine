use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

const ADD_COMPONENT_OFFSET_BORDER_RATIO: f32 = 1.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchButtonGeometryMetrics
{
    pub radius: f32,
    pub add_component_offset_y: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_geometry_metrics(
) -> WorkbenchButtonGeometryMetrics {
    button_geometry_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_geometry_metrics_from_host(
    metrics: HostControlMetrics,
) -> WorkbenchButtonGeometryMetrics {
    WorkbenchButtonGeometryMetrics {
        radius: metrics.radius_control,
        add_component_offset_y: (metrics.border_width * ADD_COMPONENT_OFFSET_BORDER_RATIO).max(0.0),
    }
}
