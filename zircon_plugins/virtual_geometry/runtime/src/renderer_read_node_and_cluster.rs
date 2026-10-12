#[cfg(test)]
use super::super::root_render_passes::VirtualGeometryNodeAndClusterCullPassStoreParts;
#[cfg(test)]
use zircon_runtime::core::framework::render::RenderVirtualGeometryNodeAndClusterCullTraversalRecord;

#[cfg(test)]
pub(crate) fn read_virtual_geometry_node_and_cluster_cull_traversal_records(
    parts: &VirtualGeometryNodeAndClusterCullPassStoreParts,
) -> Vec<RenderVirtualGeometryNodeAndClusterCullTraversalRecord> {
    parts
        .traversal_records
        .iter()
        .copied()
        .map(Into::into)
        .collect()
}
