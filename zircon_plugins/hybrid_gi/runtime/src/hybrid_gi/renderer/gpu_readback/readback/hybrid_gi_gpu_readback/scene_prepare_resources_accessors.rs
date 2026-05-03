use super::HybridGiScenePrepareResourcesSnapshot;

#[cfg_attr(not(test), allow(dead_code))]
impl HybridGiScenePrepareResourcesSnapshot {
    #[cfg(test)]
    pub(crate) fn card_capture_request_count(&self) -> u32 {
        self.card_capture_request_count
    }

    pub(crate) fn voxel_clipmap_ids(&self) -> &[u32] {
        &self.voxel_clipmap_ids
    }

    pub(crate) fn occupied_atlas_slots(&self) -> &[u32] {
        &self.occupied_atlas_slots
    }

    pub(crate) fn occupied_capture_slots(&self) -> &[u32] {
        &self.occupied_capture_slots
    }

    #[cfg(test)]
    pub(crate) fn atlas_slot_rgba_samples(&self) -> &[(u32, [u8; 4])] {
        &self.atlas_slot_rgba_samples
    }

    #[cfg(test)]
    pub(crate) fn capture_slot_rgba_samples(&self) -> &[(u32, [u8; 4])] {
        &self.capture_slot_rgba_samples
    }

    pub(crate) fn voxel_clipmap_rgba_samples(&self) -> &[(u32, [u8; 4])] {
        &self.voxel_clipmap_rgba_samples
    }

    pub(crate) fn voxel_clipmap_occupancy_masks(&self) -> &[(u32, u64)] {
        &self.voxel_clipmap_occupancy_masks
    }

    pub(crate) fn voxel_clipmap_cell_rgba_samples(&self) -> &[(u32, u32, [u8; 4])] {
        &self.voxel_clipmap_cell_rgba_samples
    }

    pub(crate) fn voxel_clipmap_cell_occupancy_counts(&self) -> &[(u32, u32, u32)] {
        &self.voxel_clipmap_cell_occupancy_counts
    }

    pub(crate) fn voxel_clipmap_cell_dominant_node_ids(&self) -> &[(u32, u32, u64)] {
        &self.voxel_clipmap_cell_dominant_node_ids
    }

    pub(crate) fn voxel_clipmap_cell_dominant_rgba_samples(&self) -> &[(u32, u32, [u8; 4])] {
        &self.voxel_clipmap_cell_dominant_rgba_samples
    }

    pub(crate) fn atlas_slot_count(&self) -> u32 {
        self.atlas_slot_count
    }

    pub(crate) fn capture_slot_count(&self) -> u32 {
        self.capture_slot_count
    }

    pub(crate) fn atlas_texture_extent(&self) -> (u32, u32) {
        self.atlas_texture_extent
    }

    pub(crate) fn capture_texture_extent(&self) -> (u32, u32) {
        self.capture_texture_extent
    }

    pub(crate) fn capture_layer_count(&self) -> u32 {
        self.capture_layer_count
    }
}
