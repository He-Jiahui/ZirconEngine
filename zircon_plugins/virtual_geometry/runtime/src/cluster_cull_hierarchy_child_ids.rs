#[cfg(test)]
use super::super::root_render_passes::VirtualGeometryNodeAndClusterCullPassStoreParts;

#[cfg(test)]
pub(crate) fn read_virtual_geometry_node_and_cluster_cull_hierarchy_child_ids(
    parts: &VirtualGeometryNodeAndClusterCullPassStoreParts,
) -> Vec<u32> {
    parts.hierarchy_child_ids.clone()
}
