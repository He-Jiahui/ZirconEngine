use super::{HybridGiProbeResidencyState, HybridGiProbeUpdateRequest, HybridGiRuntimeState};

impl HybridGiRuntimeState {
    pub(crate) fn probe_slot(&self, probe_id: u32) -> Option<u32> {
        self.resident_slots.get(&probe_id).copied()
    }

    pub(crate) fn probe_residency(&self, probe_id: u32) -> Option<HybridGiProbeResidencyState> {
        if self.resident_slots.contains_key(&probe_id) {
            return Some(HybridGiProbeResidencyState::Resident);
        }
        if self.pending_probes.contains(&probe_id) {
            return Some(HybridGiProbeResidencyState::PendingUpdate);
        }
        None
    }

    pub(crate) fn pending_updates(&self) -> Vec<HybridGiProbeUpdateRequest> {
        self.pending_updates.clone()
    }

    pub(crate) fn scheduled_trace_regions(&self) -> Vec<u32> {
        self.scheduled_trace_regions.clone()
    }

    pub(crate) fn scene_card_ids(&self) -> Vec<u32> {
        self.scene_representation.card_ids()
    }

    pub(crate) fn scene_resident_page_ids(&self) -> Vec<u32> {
        self.scene_representation.surface_cache.resident_page_ids()
    }

    pub(crate) fn scene_dirty_page_ids(&self) -> Vec<u32> {
        self.scene_representation.surface_cache.dirty_page_ids()
    }

    pub(crate) fn scene_invalidated_page_ids(&self) -> Vec<u32> {
        self.scene_representation
            .surface_cache
            .invalidated_page_ids()
    }

    pub(crate) fn scene_feedback_card_ids(&self) -> Vec<u32> {
        self.scene_representation.surface_cache.feedback_card_ids()
    }

    pub(crate) fn scene_page_table_entries(&self) -> Vec<(u32, u32)> {
        self.scene_representation.surface_cache.page_table_entries()
    }

    pub(crate) fn scene_capture_slot_entries(&self) -> Vec<(u32, u32)> {
        self.scene_representation
            .surface_cache
            .capture_slot_entries()
    }

    pub(crate) fn scene_card_capture_requests(&self) -> Vec<(u32, u32, u32, u32, [f32; 3], f32)> {
        self.scene_representation.card_capture_requests()
    }

    pub(crate) fn scene_resident_clipmap_ids(&self) -> Vec<u32> {
        self.scene_representation.voxel_scene.resident_clipmap_ids()
    }

    pub(crate) fn scene_dirty_clipmap_ids(&self) -> Vec<u32> {
        self.scene_representation.voxel_scene.dirty_clipmap_ids()
    }

    pub(crate) fn scene_invalidated_clipmap_ids(&self) -> Vec<u32> {
        self.scene_representation
            .voxel_scene
            .invalidated_clipmap_ids()
    }

    pub(crate) fn scene_clipmap_descriptors(&self) -> Vec<(u32, [f32; 3], f32)> {
        self.scene_representation.voxel_scene.clipmap_descriptors()
    }

    pub(crate) fn evictable_probes(&self) -> Vec<u32> {
        self.evictable_probes.clone()
    }

    pub(crate) fn apply_evictions(&mut self, probe_ids: impl IntoIterator<Item = u32>) {
        for probe_id in probe_ids {
            if let Some(slot) = self.resident_slots.remove(&probe_id) {
                self.free_slots.insert(slot);
            }
        }
        self.evictable_probes
            .retain(|probe_id| self.resident_slots.contains_key(probe_id));
    }

    pub(crate) fn fulfill_updates(&mut self, probe_ids: impl IntoIterator<Item = u32>) {
        for probe_id in probe_ids {
            if !self.pending_probes.remove(&probe_id) {
                continue;
            }

            self.pending_updates
                .retain(|update| update.probe_id != probe_id);
            self.promote_to_resident(probe_id);
        }
    }
}
