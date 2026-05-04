use crate::core::framework::render::FrameHistoryHandle;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct HybridGiStatSnapshot {
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
    voxel_resident_clipmap_count: usize,
    voxel_dirty_clipmap_count: usize,
    voxel_invalidated_clipmap_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct VirtualGeometryStatSnapshot {
    page_table_entry_count: usize,
    resident_page_count: usize,
    pending_request_count: usize,
    page_dependency_count: usize,
    completed_page_count: usize,
    replaced_page_count: usize,
    indirect_segment_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ParticleStatSnapshot {
    gpu_alive_count: usize,
    gpu_spawned_total: usize,
    gpu_emitter_readback_count: usize,
    gpu_indirect_draw_args: [u32; 4],
}

pub(super) struct SubmissionRecordUpdate {
    history_handle: FrameHistoryHandle,
    previous_handle: Option<FrameHistoryHandle>,
    hybrid_gi_stats: HybridGiStatSnapshot,
    particle_stats: ParticleStatSnapshot,
    virtual_geometry_stats: VirtualGeometryStatSnapshot,
}

impl HybridGiStatSnapshot {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
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
            voxel_resident_clipmap_count,
            voxel_dirty_clipmap_count,
            voxel_invalidated_clipmap_count,
        }
    }

    pub(super) fn cache_entry_count(&self) -> usize {
        self.cache_entry_count
    }

    pub(super) fn resident_probe_count(&self) -> usize {
        self.resident_probe_count
    }

    pub(super) fn pending_update_count(&self) -> usize {
        self.pending_update_count
    }

    pub(super) fn scheduled_trace_region_count(&self) -> usize {
        self.scheduled_trace_region_count
    }

    pub(super) fn scene_card_count(&self) -> usize {
        self.scene_card_count
    }

    pub(super) fn scene_screen_probe_count(&self) -> usize {
        self.scene_screen_probe_count
    }

    pub(super) fn scene_radiance_cache_entry_count(&self) -> usize {
        self.scene_radiance_cache_entry_count
    }

    pub(super) fn surface_cache_resident_page_count(&self) -> usize {
        self.surface_cache_resident_page_count
    }

    pub(super) fn surface_cache_dirty_page_count(&self) -> usize {
        self.surface_cache_dirty_page_count
    }

    pub(super) fn surface_cache_feedback_card_count(&self) -> usize {
        self.surface_cache_feedback_card_count
    }

    pub(super) fn surface_cache_capture_slot_count(&self) -> usize {
        self.surface_cache_capture_slot_count
    }

    pub(super) fn surface_cache_invalidated_page_count(&self) -> usize {
        self.surface_cache_invalidated_page_count
    }

    pub(super) fn voxel_resident_clipmap_count(&self) -> usize {
        self.voxel_resident_clipmap_count
    }

    pub(super) fn voxel_dirty_clipmap_count(&self) -> usize {
        self.voxel_dirty_clipmap_count
    }

    pub(super) fn voxel_invalidated_clipmap_count(&self) -> usize {
        self.voxel_invalidated_clipmap_count
    }
}

impl VirtualGeometryStatSnapshot {
    pub(super) fn new(
        page_table_entry_count: usize,
        resident_page_count: usize,
        pending_request_count: usize,
        page_dependency_count: usize,
        completed_page_count: usize,
        replaced_page_count: usize,
        indirect_segment_count: usize,
    ) -> Self {
        Self {
            page_table_entry_count,
            resident_page_count,
            pending_request_count,
            page_dependency_count,
            completed_page_count,
            replaced_page_count,
            indirect_segment_count,
        }
    }

    pub(super) fn page_table_entry_count(&self) -> usize {
        self.page_table_entry_count
    }

    pub(super) fn resident_page_count(&self) -> usize {
        self.resident_page_count
    }

    pub(super) fn pending_request_count(&self) -> usize {
        self.pending_request_count
    }

    pub(super) fn page_dependency_count(&self) -> usize {
        self.page_dependency_count
    }

    pub(super) fn completed_page_count(&self) -> usize {
        self.completed_page_count
    }

    pub(super) fn replaced_page_count(&self) -> usize {
        self.replaced_page_count
    }

    pub(super) fn indirect_segment_count(&self) -> usize {
        self.indirect_segment_count
    }
}

impl ParticleStatSnapshot {
    pub(super) fn new(
        gpu_alive_count: usize,
        gpu_spawned_total: usize,
        gpu_emitter_readback_count: usize,
        gpu_indirect_draw_args: [u32; 4],
    ) -> Self {
        Self {
            gpu_alive_count,
            gpu_spawned_total,
            gpu_emitter_readback_count,
            gpu_indirect_draw_args,
        }
    }

    pub(super) fn gpu_alive_count(&self) -> usize {
        self.gpu_alive_count
    }

    pub(super) fn gpu_spawned_total(&self) -> usize {
        self.gpu_spawned_total
    }

    pub(super) fn gpu_emitter_readback_count(&self) -> usize {
        self.gpu_emitter_readback_count
    }

    pub(super) fn gpu_indirect_instance_count(&self) -> usize {
        self.gpu_indirect_draw_args[1] as usize
    }
}

impl SubmissionRecordUpdate {
    pub(super) fn new(
        history_handle: FrameHistoryHandle,
        previous_handle: Option<FrameHistoryHandle>,
        hybrid_gi_stats: HybridGiStatSnapshot,
        particle_stats: ParticleStatSnapshot,
        virtual_geometry_stats: VirtualGeometryStatSnapshot,
    ) -> Self {
        Self {
            history_handle,
            previous_handle,
            hybrid_gi_stats,
            particle_stats,
            virtual_geometry_stats,
        }
    }

    pub(super) fn history_handle(&self) -> FrameHistoryHandle {
        self.history_handle
    }

    pub(super) fn previous_handle(&self) -> Option<FrameHistoryHandle> {
        self.previous_handle
    }

    pub(super) fn hybrid_gi_stats(&self) -> &HybridGiStatSnapshot {
        &self.hybrid_gi_stats
    }

    pub(super) fn particle_stats(&self) -> &ParticleStatSnapshot {
        &self.particle_stats
    }

    pub(super) fn virtual_geometry_stats(&self) -> &VirtualGeometryStatSnapshot {
        &self.virtual_geometry_stats
    }
}
