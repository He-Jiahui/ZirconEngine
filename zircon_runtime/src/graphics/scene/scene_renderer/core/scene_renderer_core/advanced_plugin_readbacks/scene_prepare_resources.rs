use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;

use super::scene_renderer_advanced_plugin_readbacks::SceneRendererAdvancedPluginReadbacks;

impl SceneRendererAdvancedPluginReadbacks {
    pub(in crate::graphics::scene::scene_renderer::core) fn hybrid_gi_scene_prepare_resources(
        &self,
    ) -> Option<&HybridGiScenePrepareResourcesSnapshot> {
        self.hybrid_gi_gpu_readback()
            .and_then(|pending| pending.scene_prepare_resources())
    }
}
