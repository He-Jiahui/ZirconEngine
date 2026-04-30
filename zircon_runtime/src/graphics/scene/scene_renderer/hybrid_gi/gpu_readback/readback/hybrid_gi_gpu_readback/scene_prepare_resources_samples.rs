use super::HybridGiScenePrepareResourcesSnapshot;

impl HybridGiScenePrepareResourcesSnapshot {
    pub(crate) fn capture_slot_rgba_sample(&self, capture_slot_id: u32) -> Option<[u8; 4]> {
        self.capture_slot_rgba_samples
            .iter()
            .find_map(|(slot_id, rgba)| (*slot_id == capture_slot_id).then_some(*rgba))
    }

    pub(crate) fn atlas_slot_rgba_sample(&self, atlas_slot_id: u32) -> Option<[u8; 4]> {
        self.atlas_slot_rgba_samples
            .iter()
            .find_map(|(slot_id, rgba)| (*slot_id == atlas_slot_id).then_some(*rgba))
    }

    pub(crate) fn voxel_clipmap_rgba_sample(&self, clipmap_id: u32) -> Option<[u8; 4]> {
        self.voxel_clipmap_rgba_samples
            .iter()
            .find_map(|(resource_clipmap_id, rgba)| {
                (*resource_clipmap_id == clipmap_id).then_some(*rgba)
            })
    }

    pub(crate) fn voxel_clipmap_cell_rgba_sample(
        &self,
        clipmap_id: u32,
        cell_index: u32,
    ) -> Option<[u8; 4]> {
        self.voxel_clipmap_cell_rgba_samples.iter().find_map(
            |(resource_clipmap_id, resource_cell_index, rgba)| {
                (*resource_clipmap_id == clipmap_id && *resource_cell_index == cell_index)
                    .then_some(*rgba)
            },
        )
    }

    pub(crate) fn voxel_clipmap_cell_dominant_rgba_sample(
        &self,
        clipmap_id: u32,
        cell_index: u32,
    ) -> Option<[u8; 4]> {
        self.voxel_clipmap_cell_dominant_rgba_samples
            .iter()
            .find_map(|(resource_clipmap_id, resource_cell_index, rgba)| {
                (*resource_clipmap_id == clipmap_id && *resource_cell_index == cell_index)
                    .then_some(*rgba)
            })
    }
}
