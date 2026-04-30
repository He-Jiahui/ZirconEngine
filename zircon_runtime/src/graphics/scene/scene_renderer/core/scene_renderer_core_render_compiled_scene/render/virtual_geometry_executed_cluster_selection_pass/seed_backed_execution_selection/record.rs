use crate::core::framework::render::RenderVirtualGeometrySelectedCluster;
use crate::graphics::types::VirtualGeometryClusterSelection;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene::render::virtual_geometry_executed_cluster_selection_pass)
struct SeedBackedExecutionSelectionRecord
{
    selection: VirtualGeometryClusterSelection,
    selected_cluster: RenderVirtualGeometrySelectedCluster,
}

impl SeedBackedExecutionSelectionRecord {
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene::render::virtual_geometry_executed_cluster_selection_pass) fn new(
        selection: VirtualGeometryClusterSelection,
        selected_cluster: RenderVirtualGeometrySelectedCluster,
    ) -> Self {
        Self {
            selection,
            selected_cluster,
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene::render::virtual_geometry_executed_cluster_selection_pass) fn selection(
        &self,
    ) -> &VirtualGeometryClusterSelection {
        &self.selection
    }

    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene::render::virtual_geometry_executed_cluster_selection_pass) fn selected_cluster(
        &self,
    ) -> &RenderVirtualGeometrySelectedCluster {
        &self.selected_cluster
    }

    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene::render::virtual_geometry_executed_cluster_selection_pass) fn selected_cluster_key(
        &self,
    ) -> (u64, u32) {
        (
            self.selected_cluster.entity,
            self.selected_cluster.cluster_id,
        )
    }

    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene::render::virtual_geometry_executed_cluster_selection_pass) fn assign_frontier_rank(
        &mut self,
        frontier_rank: u32,
    ) {
        self.selection.frontier_rank = frontier_rank;
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene::render::virtual_geometry_executed_cluster_selection_pass) fn into_selection(
        self,
    ) -> VirtualGeometryClusterSelection {
        self.selection
    }

    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene::render::virtual_geometry_executed_cluster_selection_pass) fn into_parts(
        self,
    ) -> (
        VirtualGeometryClusterSelection,
        RenderVirtualGeometrySelectedCluster,
    ) {
        (self.selection, self.selected_cluster)
    }
}
