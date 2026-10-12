use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;

use super::super::pane_data_conversion::{
    projected_sample_grid_data, projected_timeline_strip_data, projected_weight_heatmap_data,
};

pub(super) struct ProjectedTypedCanvasData {
    pub(super) sample_grid: host_contract::TemplatePaneSampleGridData,
    pub(super) timeline_strip: host_contract::TemplatePaneTimelineStripData,
    pub(super) weight_heatmap: host_contract::TemplatePaneWeightHeatmapData,
}

pub(super) fn projected_typed_canvas_data(
    component_role: &str,
    values: &BTreeMap<String, toml::Value>,
) -> ProjectedTypedCanvasData {
    ProjectedTypedCanvasData {
        sample_grid: projected_sample_grid_data(component_role, values),
        timeline_strip: projected_timeline_strip_data(component_role, values),
        weight_heatmap: projected_weight_heatmap_data(component_role, values),
    }
}
