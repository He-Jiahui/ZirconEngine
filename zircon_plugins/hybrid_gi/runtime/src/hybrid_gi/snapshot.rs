use super::declarations::HybridGiRuntimeSnapshot;
use super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(crate) fn snapshot(&self) -> HybridGiRuntimeSnapshot {
        let scene_representation = self.scene_representation();
        HybridGiRuntimeSnapshot::new(
            self.resident_probe_count(),
            self.resident_probe_count(),
            self.pending_update_count(),
            self.scheduled_trace_region_ids().len(),
            scene_representation.card_count(),
            scene_representation.surface_cache().resident_page_count(),
            scene_representation.surface_cache().dirty_page_count(),
            scene_representation.surface_cache().feedback_card_count(),
            scene_representation.card_capture_request_count(),
            scene_representation
                .surface_cache()
                .invalidated_page_count(),
            scene_representation.voxel_scene().resident_clipmap_count(),
            scene_representation.voxel_scene().dirty_clipmap_count(),
            scene_representation
                .voxel_scene()
                .invalidated_clipmap_count(),
        )
    }
}
