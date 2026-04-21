#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiRuntimeSnapshot {
    pub(crate) cache_entry_count: usize,
    pub(crate) resident_probe_count: usize,
    pub(crate) pending_update_count: usize,
    pub(crate) scheduled_trace_region_count: usize,
    pub(crate) scene_card_count: usize,
    pub(crate) surface_cache_resident_page_count: usize,
    pub(crate) surface_cache_dirty_page_count: usize,
    pub(crate) surface_cache_feedback_card_count: usize,
    pub(crate) surface_cache_capture_slot_count: usize,
    pub(crate) surface_cache_invalidated_page_count: usize,
    pub(crate) voxel_resident_clipmap_count: usize,
    pub(crate) voxel_dirty_clipmap_count: usize,
    pub(crate) voxel_invalidated_clipmap_count: usize,
}
