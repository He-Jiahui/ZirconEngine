use crate::ui::retained_host as host_contract;

use super::super::weight_heatmap::ProjectedWeightHeatmap;

pub(super) fn assign_weight_heatmap_fields(
    node: &mut host_contract::TemplatePaneNodeData,
    weight_heatmap: ProjectedWeightHeatmap,
) {
    node.weight_heatmap = weight_heatmap.data;
}
