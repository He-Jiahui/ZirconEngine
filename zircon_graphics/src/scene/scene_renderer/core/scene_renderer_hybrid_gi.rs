use crate::scene::scene_renderer::HybridGiGpuReadback;

use super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn take_last_hybrid_gi_gpu_readback(&mut self) -> Option<HybridGiGpuReadback> {
        self.last_hybrid_gi_gpu_readback.take()
    }
}
