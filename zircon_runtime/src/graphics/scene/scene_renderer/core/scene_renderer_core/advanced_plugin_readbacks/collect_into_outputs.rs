use crate::graphics::scene::scene_renderer::core::scene_renderer::SceneRendererAdvancedPluginOutputs;
use crate::graphics::types::GraphicsError;

use super::scene_renderer_advanced_plugin_readbacks::SceneRendererAdvancedPluginReadbacks;

impl SceneRendererAdvancedPluginReadbacks {
    pub(in crate::graphics::scene::scene_renderer::core) fn collect_into_outputs(
        self,
        device: &wgpu::Device,
        outputs: &mut SceneRendererAdvancedPluginOutputs,
    ) -> Result<(), GraphicsError> {
        let (hybrid_gi_gpu_readback, virtual_geometry_gpu_readback) = self.into_pending_readbacks();

        outputs.store_hybrid_gi_gpu_readback(
            hybrid_gi_gpu_readback
                .map(|pending| pending.collect(device))
                .transpose()?,
        );
        outputs.store_virtual_geometry_gpu_readback(
            virtual_geometry_gpu_readback
                .map(|pending| pending.collect(device))
                .transpose()?,
        );
        Ok(())
    }
}
