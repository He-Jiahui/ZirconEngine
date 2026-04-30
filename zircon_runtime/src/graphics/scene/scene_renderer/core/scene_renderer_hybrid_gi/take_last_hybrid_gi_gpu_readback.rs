use crate::graphics::scene::scene_renderer::HybridGiGpuReadback;

use super::super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn take_last_hybrid_gi_gpu_readback(&mut self) -> Option<HybridGiGpuReadback> {
        self.advanced_plugin_outputs.take_hybrid_gi_gpu_readback()
    }
}
