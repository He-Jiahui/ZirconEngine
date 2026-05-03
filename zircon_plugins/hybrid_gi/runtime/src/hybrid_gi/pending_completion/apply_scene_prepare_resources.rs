use super::super::HybridGiRuntimeState;
use super::super::HybridGiScenePrepareResourceSamples;

impl HybridGiRuntimeState {
    pub(crate) fn apply_scene_prepare_resources(
        &mut self,
        resources: &dyn HybridGiScenePrepareResourceSamples,
    ) {
        self.scene_representation_mut()
            .surface_cache_mut()
            .apply_scene_prepare_resources(resources);
        let surface_cache_page_contents = self
            .scene_representation()
            .surface_cache()
            .page_contents_snapshot();
        let voxel_cells = resources.voxel_cells();
        let voxel_scene = self.scene_representation_mut().voxel_scene_mut();
        voxel_scene.apply_surface_cache_page_contents(&surface_cache_page_contents);
        voxel_scene.apply_scene_prepare_voxel_cells(voxel_cells);
    }
}
