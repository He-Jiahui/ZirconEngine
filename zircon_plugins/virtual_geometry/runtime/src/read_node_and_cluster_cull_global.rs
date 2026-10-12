#[cfg(test)]
use super::super::root_render_passes::VirtualGeometryNodeAndClusterCullPassStoreParts;
#[cfg(test)]
use zircon_runtime::core::framework::render::RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot;

#[cfg(test)]
pub(crate) fn read_virtual_geometry_node_and_cluster_cull_global_state_snapshot(
    parts: &VirtualGeometryNodeAndClusterCullPassStoreParts,
) -> Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot> {
    parts.global_state.clone()
}
