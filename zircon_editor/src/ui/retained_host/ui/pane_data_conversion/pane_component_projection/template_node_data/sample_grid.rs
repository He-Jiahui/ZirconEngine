use crate::ui::retained_host as host_contract;

use super::super::sample_grid::ProjectedSampleGrid;

pub(super) fn assign_sample_grid_fields(
    node: &mut host_contract::TemplatePaneNodeData,
    sample_grid: ProjectedSampleGrid,
) {
    node.sample_grid = sample_grid.data;
}
