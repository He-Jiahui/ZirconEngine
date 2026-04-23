use crate::graphics::scene::HybridGiScenePrepareResourcesSnapshot;

use super::super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(crate) fn apply_scene_prepare_resources(
        &mut self,
        snapshot: &HybridGiScenePrepareResourcesSnapshot,
    ) {
        self.scene_representation
            .surface_cache
            .apply_scene_prepare_resources(snapshot);
        let surface_cache_page_contents = self
            .scene_representation
            .surface_cache
            .page_contents_snapshot();
        self.scene_representation
            .voxel_scene
            .apply_surface_cache_page_contents(&surface_cache_page_contents);
    }
}
