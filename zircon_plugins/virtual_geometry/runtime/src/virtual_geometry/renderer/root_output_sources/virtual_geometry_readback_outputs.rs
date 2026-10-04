use crate::virtual_geometry::renderer::{
    VirtualGeometryGpuReadback, VirtualGeometryGpuReadbackCompletionParts,
};
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryNodeClusterCullReadbackOutputs, RenderVirtualGeometryReadbackOutputs,
};

#[derive(Default)]
pub(super) struct VirtualGeometryReadbackOutputs {
    gpu_readback: Option<VirtualGeometryGpuReadback>,
}

impl VirtualGeometryReadbackOutputs {
    pub(in crate::virtual_geometry::renderer) fn store_gpu_readback(
        &mut self,
        readback: Option<VirtualGeometryGpuReadback>,
    ) {
        self.gpu_readback = readback;
    }

    pub(in crate::virtual_geometry::renderer) fn store_node_cluster_cull_readback(
        &mut self,
        node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs,
    ) {
        self.gpu_readback
            .get_or_insert_with(|| {
                VirtualGeometryGpuReadback::new(Vec::new(), Vec::new(), Vec::new(), Vec::new())
            })
            .replace_node_cluster_cull_readback(node_cluster_cull);
    }

    pub(in crate::virtual_geometry::renderer) fn has_gpu_readback(&self) -> bool {
        self.gpu_readback.is_some()
    }

    pub(in crate::virtual_geometry::renderer) fn gpu_readback_mut(
        &mut self,
    ) -> Option<&mut VirtualGeometryGpuReadback> {
        self.gpu_readback.as_mut()
    }

    pub(in crate::virtual_geometry::renderer) fn gpu_readback(
        &self,
    ) -> Option<&VirtualGeometryGpuReadback> {
        self.gpu_readback.as_ref()
    }

    // 完成部件和中立输出都通过 take 消费同一份 GPU 读回，避免后续阶段重复发布。
    pub(in crate::virtual_geometry::renderer) fn take_gpu_completion_parts(
        &mut self,
    ) -> Option<VirtualGeometryGpuReadbackCompletionParts> {
        self.gpu_readback
            .take()
            .map(VirtualGeometryGpuReadback::into_completion_parts)
    }

    pub(in crate::virtual_geometry::renderer) fn take_neutral_readback_outputs(
        &mut self,
    ) -> RenderVirtualGeometryReadbackOutputs {
        self.gpu_readback
            .take()
            .map(RenderVirtualGeometryReadbackOutputs::from)
            .unwrap_or_default()
    }

    #[cfg(test)]
    pub(crate) fn take_gpu_readback(&mut self) -> Option<VirtualGeometryGpuReadback> {
        self.gpu_readback.take()
    }
}

#[cfg(test)]
#[path = "tests/virtual_geometry_readback_outputs.rs"]
mod tests;
