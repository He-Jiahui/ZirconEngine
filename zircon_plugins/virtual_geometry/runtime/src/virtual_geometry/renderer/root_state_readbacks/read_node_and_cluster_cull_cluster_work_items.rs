#[cfg(test)]
use super::super::root_render_passes::VirtualGeometryNodeAndClusterCullPassStoreParts;
#[cfg(test)]
use zircon_runtime::core::framework::render::RenderVirtualGeometryNodeAndClusterCullClusterWorkItem;

#[cfg(test)]
pub(crate) fn read_virtual_geometry_node_and_cluster_cull_cluster_work_items(
    parts: &VirtualGeometryNodeAndClusterCullPassStoreParts,
) -> Vec<RenderVirtualGeometryNodeAndClusterCullClusterWorkItem> {
    parts
        .cluster_work_items
        .iter()
        .copied()
        .map(Into::into)
        .collect()
}
