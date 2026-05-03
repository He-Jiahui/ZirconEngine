use crate::core::framework::render::RenderHybridGiReadbackOutputs;

use super::super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub(in crate::graphics) fn take_last_hybrid_gi_readback_outputs(
        &mut self,
    ) -> RenderHybridGiReadbackOutputs {
        self.advanced_plugin_outputs
            .take_hybrid_gi_readback_outputs()
    }
}
