use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;

use super::HybridGiGpuPendingReadback;

impl HybridGiGpuPendingReadback {
    pub(in crate::graphics::scene::scene_renderer) fn scene_prepare_resources(
        &self,
    ) -> Option<&HybridGiScenePrepareResourcesSnapshot> {
        self.scene_prepare_resources.as_ref()
    }
}
