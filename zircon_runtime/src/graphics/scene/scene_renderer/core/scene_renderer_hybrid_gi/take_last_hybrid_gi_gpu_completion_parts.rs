use crate::graphics::scene::scene_renderer::HybridGiGpuReadbackCompletionParts;

use super::super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub(in crate::graphics) fn take_last_hybrid_gi_gpu_completion_parts(
        &mut self,
    ) -> Option<HybridGiGpuReadbackCompletionParts> {
        self.advanced_plugin_outputs
            .take_hybrid_gi_gpu_completion_parts()
    }
}
