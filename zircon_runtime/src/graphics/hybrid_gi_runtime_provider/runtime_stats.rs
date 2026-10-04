use crate::core::framework::render::{
    RenderHybridGiGlobalSdfStats, RenderHybridGiResolvedSettings,
    RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HybridGiRuntimeStats {
    cache_entry_count: usize,
    resident_probe_count: usize,
    pending_update_count: usize,
    scheduled_trace_region_count: usize,
    scene_card_count: usize,
    scene_screen_probe_count: usize,
    scene_radiance_cache_entry_count: usize,
    radiance_cache_resident_probe_count: usize,
    radiance_cache_update_probe_count: usize,
    radiance_cache_truncated_demand_count: usize,
    radiance_cache_generation: u64,
    radiance_cache_scroll_count: u64,
    radiance_cache_history_clear_count: u64,
    radiance_cache_gpu_stage_dispatch_counts:
        [u32; RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT],
    global_sdf_stats: RenderHybridGiGlobalSdfStats,
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
        radiance_cache_resident_probe_count: usize,
        radiance_cache_update_probe_count: usize,
        radiance_cache_truncated_demand_count: usize,
        radiance_cache_generation: u64,
        radiance_cache_scroll_count: u64,
        radiance_cache_history_clear_count: u64,
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
            radiance_cache_resident_probe_count,
            radiance_cache_update_probe_count,
            radiance_cache_truncated_demand_count,
            radiance_cache_generation,
            radiance_cache_scroll_count,
            radiance_cache_history_clear_count,
            radiance_cache_gpu_stage_dispatch_counts: Default::default(),
            global_sdf_stats: Default::default(),
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

    pub fn with_radiance_cache_gpu_stage_dispatch_counts(
        mut self,
        counts: [u32; RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT],
    ) -> Self {
        self.radiance_cache_gpu_stage_dispatch_counts = counts;
        self
    }

    pub fn with_global_sdf_stats(mut self, stats: RenderHybridGiGlobalSdfStats) -> Self {
        self.global_sdf_stats = stats;
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

    pub fn radiance_cache_resident_probe_count(&self) -> usize {
        self.radiance_cache_resident_probe_count
    }

    pub fn radiance_cache_update_probe_count(&self) -> usize {
        self.radiance_cache_update_probe_count
    }

    pub fn radiance_cache_truncated_demand_count(&self) -> usize {
        self.radiance_cache_truncated_demand_count
    }

    pub fn radiance_cache_generation(&self) -> u64 {
        self.radiance_cache_generation
    }

    pub fn radiance_cache_scroll_count(&self) -> u64 {
        self.radiance_cache_scroll_count
    }

    pub fn radiance_cache_history_clear_count(&self) -> u64 {
        self.radiance_cache_history_clear_count
    }

    pub fn radiance_cache_gpu_stage_dispatch_counts(
        &self,
    ) -> [u32; RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT] {
        self.radiance_cache_gpu_stage_dispatch_counts
    }

    pub fn global_sdf_stats(&self) -> RenderHybridGiGlobalSdfStats {
        self.global_sdf_stats
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
