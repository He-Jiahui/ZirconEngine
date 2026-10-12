#[cfg(test)]
use super::super::root_render_passes::VirtualGeometryNodeAndClusterCullPassStoreParts;
#[cfg(test)]
use zircon_runtime::core::framework::render::RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem;

#[cfg(test)]
pub(crate) fn read_virtual_geometry_node_and_cluster_cull_instance_work_items(
    parts: &VirtualGeometryNodeAndClusterCullPassStoreParts,
) -> Vec<RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem> {
    parts.instance_work_items.clone()
}
