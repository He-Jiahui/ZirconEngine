use super::HybridGiScenePrepareResourcesSnapshot;

impl HybridGiScenePrepareResourcesSnapshot {
    pub(in crate::graphics::scene::scene_renderer::hybrid_gi::gpu_readback::readback) fn into_surface_cache_samples(
        self,
    ) -> (Vec<(u32, [u8; 4])>, Vec<(u32, [u8; 4])>) {
        (self.atlas_slot_rgba_samples, self.capture_slot_rgba_samples)
    }
}
