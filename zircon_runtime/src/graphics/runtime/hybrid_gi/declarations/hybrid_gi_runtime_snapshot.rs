#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiRuntimeSnapshot {
    cache_entry_count: usize,
    resident_probe_count: usize,
    pending_update_count: usize,
    scheduled_trace_region_count: usize,
    scene_card_count: usize,
    surface_cache_resident_page_count: usize,
    surface_cache_dirty_page_count: usize,
    surface_cache_feedback_card_count: usize,
    surface_cache_capture_slot_count: usize,
    surface_cache_invalidated_page_count: usize,
    voxel_resident_clipmap_count: usize,
    voxel_dirty_clipmap_count: usize,
    voxel_invalidated_clipmap_count: usize,
}

impl HybridGiRuntimeSnapshot {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics::runtime::hybrid_gi) fn new(
        cache_entry_count: usize,
        resident_probe_count: usize,
        pending_update_count: usize,
        scheduled_trace_region_count: usize,
        scene_card_count: usize,
        surface_cache_resident_page_count: usize,
        surface_cache_dirty_page_count: usize,
        surface_cache_feedback_card_count: usize,
        surface_cache_capture_slot_count: usize,
        surface_cache_invalidated_page_count: usize,
        voxel_resident_clipmap_count: usize,
        voxel_dirty_clipmap_count: usize,
        voxel_invalidated_clipmap_count: usize,
    ) -> Self {
        Self {
            cache_entry_count,
            resident_probe_count,
            pending_update_count,
            scheduled_trace_region_count,
            scene_card_count,
            surface_cache_resident_page_count,
            surface_cache_dirty_page_count,
            surface_cache_feedback_card_count,
            surface_cache_capture_slot_count,
            surface_cache_invalidated_page_count,
            voxel_resident_clipmap_count,
            voxel_dirty_clipmap_count,
            voxel_invalidated_clipmap_count,
        }
    }

    pub(crate) fn cache_entry_count(&self) -> usize {
        self.cache_entry_count
    }

    pub(crate) fn resident_probe_count(&self) -> usize {
        self.resident_probe_count
    }

    pub(crate) fn pending_update_count(&self) -> usize {
        self.pending_update_count
    }

    pub(crate) fn scheduled_trace_region_count(&self) -> usize {
        self.scheduled_trace_region_count
    }

    pub(crate) fn scene_card_count(&self) -> usize {
        self.scene_card_count
    }

    pub(crate) fn surface_cache_resident_page_count(&self) -> usize {
        self.surface_cache_resident_page_count
    }

    pub(crate) fn surface_cache_dirty_page_count(&self) -> usize {
        self.surface_cache_dirty_page_count
    }

    pub(crate) fn surface_cache_feedback_card_count(&self) -> usize {
        self.surface_cache_feedback_card_count
    }

    pub(crate) fn surface_cache_capture_slot_count(&self) -> usize {
        self.surface_cache_capture_slot_count
    }

    pub(crate) fn surface_cache_invalidated_page_count(&self) -> usize {
        self.surface_cache_invalidated_page_count
    }

    pub(crate) fn voxel_resident_clipmap_count(&self) -> usize {
        self.voxel_resident_clipmap_count
    }

    pub(crate) fn voxel_dirty_clipmap_count(&self) -> usize {
        self.voxel_dirty_clipmap_count
    }

    pub(crate) fn voxel_invalidated_clipmap_count(&self) -> usize {
        self.voxel_invalidated_clipmap_count
    }
}
