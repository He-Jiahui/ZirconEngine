use super::VirtualGeometryGpuReadback;
use zircon_runtime::core::framework::render::RenderVirtualGeometryNodeClusterCullReadbackOutputs;

impl VirtualGeometryGpuReadback {
    pub(crate) fn replace_node_cluster_cull_readback(
        &mut self,
        node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs,
    ) {
        self.node_cluster_cull = node_cluster_cull;
    }
}
