use crate::core::framework::render::RenderHybridGiResolvedSettings;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HybridGiRuntimeStats {
    cache_entry_count: usize,
    resident_probe_count: usize,
    pending_update_count: usize,
    scheduled_trace_region_count: usize,
    scene_card_count: usize,
    scene_screen_probe_count: usize,
    scene_radiance_cache_entry_count: usize,
    surface_cache_resident_page_count: usize,
    surface_cache_dirty_page_count: usize,
    surface_cache_feedback_card_count: usize,
    surface_cache_capture_slot_count: usize,
    surface_cache_invalidated_page_count: usize,
    surface_cache_depth_sample_count: usize,
    probe_trace_tile_count: usize,
    probe_trace_dispatch_group_count: [usize; 3],
    voxel_resident_clipmap_count: usize,
    voxel_dirty_clipmap_count: usize,
    voxel_invalidated_clipmap_count: usize,
    resolved_settings: Option<RenderHybridGiResolvedSettings>,
}

impl HybridGiRuntimeStats {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cache_entry_count: usize,
        resident_probe_count: usize,
        pending_update_count: usize,
        scheduled_trace_region_count: usize,
        scene_card_count: usize,
        scene_screen_probe_count: usize,
        scene_radiance_cache_entry_count: usize,
        surface_cache_resident_page_count: usize,
        surface_cache_dirty_page_count: usize,
        surface_cache_feedback_card_count: usize,
        surface_cache_capture_slot_count: usize,
        surface_cache_invalidated_page_count: usize,
        surface_cache_depth_sample_count: usize,
        probe_trace_tile_count: usize,
        probe_trace_dispatch_group_count: [usize; 3],
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
            scene_screen_probe_count,
            scene_radiance_cache_entry_count,
            surface_cache_resident_page_count,
            surface_cache_dirty_page_count,
            surface_cache_feedback_card_count,
            surface_cache_capture_slot_count,
            surface_cache_invalidated_page_count,
            surface_cache_depth_sample_count,
            probe_trace_tile_count,
            probe_trace_dispatch_group_count,
            voxel_resident_clipmap_count,
            voxel_dirty_clipmap_count,
            voxel_invalidated_clipmap_count,
            resolved_settings: None,
        }
    }

    pub fn with_resolved_settings(
        mut self,
        resolved_settings: Option<RenderHybridGiResolvedSettings>,
    ) -> Self {
        self.resolved_settings = resolved_settings;
        self
    }

    pub fn cache_entry_count(&self) -> usize {
        self.cache_entry_count
    }

    pub fn resident_probe_count(&self) -> usize {
        self.resident_probe_count
    }

    pub fn pending_update_count(&self) -> usize {
        self.pending_update_count
    }

    pub fn scheduled_trace_region_count(&self) -> usize {
        self.scheduled_trace_region_count
    }

    pub fn scene_card_count(&self) -> usize {
        self.scene_card_count
    }

    pub fn scene_screen_probe_count(&self) -> usize {
        self.scene_screen_probe_count
    }

    pub fn scene_radiance_cache_entry_count(&self) -> usize {
        self.scene_radiance_cache_entry_count
    }

    pub fn surface_cache_resident_page_count(&self) -> usize {
        self.surface_cache_resident_page_count
    }

    pub fn surface_cache_dirty_page_count(&self) -> usize {
        self.surface_cache_dirty_page_count
    }

    pub fn surface_cache_feedback_card_count(&self) -> usize {
        self.surface_cache_feedback_card_count
    }

    pub fn surface_cache_capture_slot_count(&self) -> usize {
        self.surface_cache_capture_slot_count
    }

    pub fn surface_cache_invalidated_page_count(&self) -> usize {
        self.surface_cache_invalidated_page_count
    }

    pub fn surface_cache_depth_sample_count(&self) -> usize {
        self.surface_cache_depth_sample_count
    }

    pub fn probe_trace_tile_count(&self) -> usize {
        self.probe_trace_tile_count
    }

    pub fn probe_trace_dispatch_group_count(&self) -> [usize; 3] {
        self.probe_trace_dispatch_group_count
    }

    pub fn voxel_resident_clipmap_count(&self) -> usize {
        self.voxel_resident_clipmap_count
    }

    pub fn voxel_dirty_clipmap_count(&self) -> usize {
        self.voxel_dirty_clipmap_count
    }

    pub fn voxel_invalidated_clipmap_count(&self) -> usize {
        self.voxel_invalidated_clipmap_count
    }

    pub fn resolved_settings(&self) -> Option<RenderHybridGiResolvedSettings> {
        self.resolved_settings
    }
}
