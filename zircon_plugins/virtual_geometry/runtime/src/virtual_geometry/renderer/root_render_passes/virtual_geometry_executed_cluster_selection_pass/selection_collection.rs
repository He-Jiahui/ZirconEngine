use crate::virtual_geometry::types::VirtualGeometryClusterSelection;
use zircon_runtime::core::framework::render::RenderVirtualGeometrySelectedCluster;

use super::seed_backed_execution_selection::SeedBackedExecutionSelectionRecord;

#[derive(Default)]
pub(super) struct ExecutedClusterSelectionCollection {
    selections: Vec<VirtualGeometryClusterSelection>,
    selected_clusters: Vec<RenderVirtualGeometrySelectedCluster>,
}

impl ExecutedClusterSelectionCollection {
    pub(super) fn from_selections(selections: Vec<VirtualGeometryClusterSelection>) -> Self {
        let selected_clusters = selections
            .iter()
            .copied()
            .map(VirtualGeometryClusterSelection::to_selected_cluster)
            .collect();
        Self {
            selections,
            selected_clusters,
        }
    }

    pub(super) fn from_seed_backed_records(
        records: Vec<SeedBackedExecutionSelectionRecord>,
    ) -> Self {
        let (selections, selected_clusters) = records
            .into_iter()
            .map(SeedBackedExecutionSelectionRecord::into_parts)
            .unzip();
        Self {
            selections,
            selected_clusters,
        }
    }

    #[cfg(test)]
    pub(super) fn selections(&self) -> &[VirtualGeometryClusterSelection] {
        &self.selections
    }

    pub(super) fn selected_clusters(&self) -> &[RenderVirtualGeometrySelectedCluster] {
        &self.selected_clusters
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        Vec<VirtualGeometryClusterSelection>,
        Vec<RenderVirtualGeometrySelectedCluster>,
    ) {
        (self.selections, self.selected_clusters)
    }
}
