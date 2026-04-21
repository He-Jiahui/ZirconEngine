use super::declarations::HybridGiRuntimeSnapshot;
use super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(crate) fn snapshot(&self) -> HybridGiRuntimeSnapshot {
        HybridGiRuntimeSnapshot {
            cache_entry_count: self.resident_slots.len(),
            resident_probe_count: self.resident_slots.len(),
            pending_update_count: self.pending_updates.len(),
            scheduled_trace_region_count: self.scheduled_trace_regions.len(),
            scene_card_count: self.scene_representation.card_count(),
            surface_cache_resident_page_count: self
                .scene_representation
                .surface_cache
                .resident_page_count(),
            surface_cache_dirty_page_count: self
                .scene_representation
                .surface_cache
                .dirty_page_count(),
            surface_cache_feedback_card_count: self
                .scene_representation
                .surface_cache
                .feedback_card_count(),
            surface_cache_capture_slot_count: self
                .scene_representation
                .card_capture_request_count(),
            surface_cache_invalidated_page_count: self
                .scene_representation
                .surface_cache
                .invalidated_page_count(),
            voxel_resident_clipmap_count: self
                .scene_representation
                .voxel_scene
                .resident_clipmap_count(),
            voxel_dirty_clipmap_count: self.scene_representation.voxel_scene.dirty_clipmap_count(),
            voxel_invalidated_clipmap_count: self
                .scene_representation
                .voxel_scene
                .invalidated_clipmap_count(),
        }
    }
}
