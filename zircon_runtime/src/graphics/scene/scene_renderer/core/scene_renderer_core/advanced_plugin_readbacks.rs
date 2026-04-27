use super::super::super::{
    HybridGiGpuPendingReadback, HybridGiScenePrepareResourcesSnapshot,
    VirtualGeometryGpuPendingReadback,
};
use super::super::scene_renderer::SceneRendererAdvancedPluginOutputs;
use crate::graphics::types::GraphicsError;

pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererAdvancedPluginReadbacks {
    pub(super) hybrid_gi_gpu_readback: Option<HybridGiGpuPendingReadback>,
    pub(super) virtual_geometry_gpu_readback: Option<VirtualGeometryGpuPendingReadback>,
}

impl SceneRendererAdvancedPluginReadbacks {
    pub(in crate::graphics::scene::scene_renderer::core) fn hybrid_gi_scene_prepare_resources(
        &self,
    ) -> Option<&HybridGiScenePrepareResourcesSnapshot> {
        self.hybrid_gi_gpu_readback
            .as_ref()
            .and_then(|pending| pending.scene_prepare_resources())
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn collect_into_outputs(
        self,
        device: &wgpu::Device,
        outputs: &mut SceneRendererAdvancedPluginOutputs,
    ) -> Result<(), GraphicsError> {
        outputs.hybrid_gi_gpu_readback = self
            .hybrid_gi_gpu_readback
            .map(|pending| pending.collect(device))
            .transpose()?;
        outputs.virtual_geometry_gpu_readback = self
            .virtual_geometry_gpu_readback
            .map(|pending| pending.collect(device))
            .transpose()?;
        Ok(())
    }
}
