#[cfg(test)]
use super::super::root_render_passes::VirtualGeometryNodeAndClusterCullPassStoreParts;
#[cfg(test)]
use zircon_runtime::core::framework::render::RenderVirtualGeometryNodeAndClusterCullSource;

#[cfg(test)]
pub(crate) fn read_virtual_geometry_node_and_cluster_cull_source(
    parts: &VirtualGeometryNodeAndClusterCullPassStoreParts,
) -> RenderVirtualGeometryNodeAndClusterCullSource {
    parts.source
}
