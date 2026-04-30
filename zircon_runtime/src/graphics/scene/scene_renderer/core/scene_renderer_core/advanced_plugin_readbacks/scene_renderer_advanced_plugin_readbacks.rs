use crate::graphics::scene::scene_renderer::{
    HybridGiGpuPendingReadback, VirtualGeometryGpuPendingReadback,
};

pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererAdvancedPluginReadbacks {
    hybrid_gi_gpu_readback: Option<HybridGiGpuPendingReadback>,
    virtual_geometry_gpu_readback: Option<VirtualGeometryGpuPendingReadback>,
}

impl SceneRendererAdvancedPluginReadbacks {
    pub(in crate::graphics::scene::scene_renderer::core) fn new(
        hybrid_gi_gpu_readback: Option<HybridGiGpuPendingReadback>,
        virtual_geometry_gpu_readback: Option<VirtualGeometryGpuPendingReadback>,
    ) -> Self {
        Self {
            hybrid_gi_gpu_readback,
            virtual_geometry_gpu_readback,
        }
    }

    pub(super) fn hybrid_gi_gpu_readback(&self) -> Option<&HybridGiGpuPendingReadback> {
        self.hybrid_gi_gpu_readback.as_ref()
    }

    pub(super) fn into_pending_readbacks(
        self,
    ) -> (
        Option<HybridGiGpuPendingReadback>,
        Option<VirtualGeometryGpuPendingReadback>,
    ) {
        (
            self.hybrid_gi_gpu_readback,
            self.virtual_geometry_gpu_readback,
        )
    }
}
