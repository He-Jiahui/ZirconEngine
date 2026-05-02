use std::sync::Arc;

use crate::virtual_geometry::types::VirtualGeometryClusterSelection;
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometrySelectedCluster, RenderVirtualGeometrySelectedClusterSource,
};

#[derive(Default)]
pub(in crate::virtual_geometry::renderer::root_render_passes) struct VirtualGeometryExecutedClusterSelectionPassOutput
{
    selections: Vec<VirtualGeometryClusterSelection>,
    selected_clusters: Vec<RenderVirtualGeometrySelectedCluster>,
    source: RenderVirtualGeometrySelectedClusterSource,
    selected_cluster_count: u32,
    selected_cluster_buffer: Option<Arc<wgpu::Buffer>>,
}

impl VirtualGeometryExecutedClusterSelectionPassOutput {
    pub(super) fn new(
        selections: Vec<VirtualGeometryClusterSelection>,
        selected_clusters: Vec<RenderVirtualGeometrySelectedCluster>,
        source: RenderVirtualGeometrySelectedClusterSource,
        selected_cluster_count: u32,
        selected_cluster_buffer: Option<Arc<wgpu::Buffer>>,
    ) -> Self {
        Self {
            selections,
            selected_clusters,
            source,
            selected_cluster_count,
            selected_cluster_buffer,
        }
    }

    pub(in crate::virtual_geometry::renderer::root_render_passes) fn selections(
        &self,
    ) -> &[VirtualGeometryClusterSelection] {
        &self.selections
    }

    pub(in crate::virtual_geometry::renderer::root_render_passes) fn selected_clusters(
        &self,
    ) -> &[RenderVirtualGeometrySelectedCluster] {
        &self.selected_clusters
    }

    pub(in crate::virtual_geometry::renderer::root_render_passes) fn source(
        &self,
    ) -> RenderVirtualGeometrySelectedClusterSource {
        self.source
    }

    pub(in crate::virtual_geometry::renderer::root_render_passes) fn into_indirect_stats_parts(
        self,
    ) -> (
        Vec<RenderVirtualGeometrySelectedCluster>,
        RenderVirtualGeometrySelectedClusterSource,
        u32,
        Option<Arc<wgpu::Buffer>>,
    ) {
        (
            self.selected_clusters,
            self.source,
            self.selected_cluster_count,
            self.selected_cluster_buffer,
        )
    }

    #[cfg(test)]
    pub(in crate::virtual_geometry::renderer::root_render_passes) fn from_test_parts(
        selections: Vec<VirtualGeometryClusterSelection>,
        selected_clusters: Vec<RenderVirtualGeometrySelectedCluster>,
        source: RenderVirtualGeometrySelectedClusterSource,
        selected_cluster_count: u32,
        selected_cluster_buffer: Option<Arc<wgpu::Buffer>>,
    ) -> Self {
        Self {
            selections,
            selected_clusters,
            source,
            selected_cluster_count,
            selected_cluster_buffer,
        }
    }
}
