#[cfg(test)]
use super::super::root_render_passes::VirtualGeometryNodeAndClusterCullPassStoreParts;
#[cfg(test)]
use zircon_runtime::core::framework::render::RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot;

#[cfg(test)]
pub(crate) fn read_virtual_geometry_node_and_cluster_cull_launch_worklist_snapshot(
    parts: &VirtualGeometryNodeAndClusterCullPassStoreParts,
) -> Option<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot> {
    parts.launch_worklist.clone()
}
