use crate::hybrid_gi::renderer::HybridGiScenePrepareResourcesSnapshot;

use super::HybridGiGpuPendingReadback;

impl HybridGiGpuPendingReadback {
    pub(in crate::hybrid_gi::renderer) fn scene_prepare_resources(
        &self,
    ) -> Option<&HybridGiScenePrepareResourcesSnapshot> {
        self.scene_prepare_resources.as_ref()
    }
}
