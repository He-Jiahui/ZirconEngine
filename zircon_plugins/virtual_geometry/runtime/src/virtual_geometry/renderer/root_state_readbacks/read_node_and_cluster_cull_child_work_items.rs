#[cfg(test)]
use super::super::root_render_passes::VirtualGeometryNodeAndClusterCullPassStoreParts;
#[cfg(test)]
use zircon_runtime::core::framework::render::RenderVirtualGeometryNodeAndClusterCullChildWorkItem;

#[cfg(test)]
pub(crate) fn read_virtual_geometry_node_and_cluster_cull_child_work_items(
    parts: &VirtualGeometryNodeAndClusterCullPassStoreParts,
) -> Vec<RenderVirtualGeometryNodeAndClusterCullChildWorkItem> {
    parts
        .child_work_items
        .iter()
        .copied()
        .map(Into::into)
        .collect()
}
