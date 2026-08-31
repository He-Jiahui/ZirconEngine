pub const RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT: usize = 6;
pub const RENDER_HYBRID_GI_PROBE_TRACE_DIAGNOSTIC_WORD_COUNT: usize = 13;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderHybridGiRadianceCacheGpuStage {
    Mark,
    Allocate,
    Trace,
    Filter,
    BorderMip,
    Consume,
}

impl RenderHybridGiRadianceCacheGpuStage {
    pub const ALL: [Self; RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT] = [
        Self::Mark,
        Self::Allocate,
        Self::Trace,
        Self::Filter,
        Self::BorderMip,
        Self::Consume,
    ];

    pub const fn index(self) -> usize {
        match self {
            Self::Mark => 0,
            Self::Allocate => 1,
            Self::Trace => 2,
            Self::Filter => 3,
            Self::BorderMip => 4,
            Self::Consume => 5,
        }
    }
}

/// Bounded Global SDF scheduler and resource metrics emitted by the active renderer instance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiGlobalSdfStats {
    pub cpu_prepare_time_us: u64,
    pub cpu_mesh_object_collection_time_us: u64,
    pub cpu_mesh_scene_sync_time_us: u64,
    pub cpu_residency_time_us: u64,
    pub cpu_influence_update_time_us: u64,
    pub cpu_candidate_build_time_us: u64,
    /// True only when the authoritative-static Mesh SDF projection cache supplied this frame.
    pub mesh_projection_cache_hit: bool,
    pub object_count: usize,
    pub resident_page_count: usize,
    pub sampleable_page_count: usize,
    pub dirty_page_count: usize,
    pub dispatched_page_count: usize,
    pub uploaded_page_count: usize,
    pub deferred_page_count: usize,
    pub candidate_overflow_page_count: usize,
    /// Entries retained for materializable page candidate lists; terminal-overflow pages and
    /// clipmap-level typed-fallback pages are excluded.
    pub candidate_contributor_count: usize,
    pub clipmap_fallback_count: usize,
    /// Capacity retained by page candidate `Vec<u64>` allocations, excluding map metadata.
    pub candidate_bucket_capacity_bytes: u64,
    pub persistent_resource_byte_count: u64,
    pub transient_buffer_creation_count: usize,
    pub transient_bind_group_creation_count: usize,
    pub transient_parameter_upload_byte_count: u64,
    pub transient_page_upload_byte_count: u64,
    pub transient_mesh_upload_byte_count: u64,
    pub transient_completion_upload_byte_count: u64,
    pub transient_upload_byte_count: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiReadbackOutputs {
    pub cache_entries: Vec<RenderHybridGiCacheEntryRecord>,
    pub completed_probe_ids: Vec<u32>,
    pub completed_trace_region_ids: Vec<u32>,
    pub probe_irradiance_rgb: Vec<[u16; 3]>,
    pub probe_rt_lighting_rgb: Vec<[u16; 3]>,
    pub radiance_cache_gpu_stage_dispatch_counts:
        [u32; RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT],
    /// Present even for an all-zero frame so runtime statistics can clear stale values.
    pub global_sdf_stats: Option<RenderHybridGiGlobalSdfStats>,
    pub scene_prepare: RenderHybridGiScenePrepareReadbackOutputs,
}

impl RenderHybridGiReadbackOutputs {
    pub fn is_empty(&self) -> bool {
        self.cache_entries.is_empty()
            && self.completed_probe_ids.is_empty()
            && self.completed_trace_region_ids.is_empty()
            && self.probe_irradiance_rgb.is_empty()
            && self.probe_rt_lighting_rgb.is_empty()
            && self
                .radiance_cache_gpu_stage_dispatch_counts
                .iter()
                .all(|count| *count == 0)
            && self.global_sdf_stats.is_none()
            && !self.scene_prepare.has_runtime_feedback_payload()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiCacheEntryRecord {
    pub key: u64,
    pub value: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiScenePrepareReadbackOutputs {
    pub occupied_atlas_slots: Vec<u32>,
    pub occupied_capture_slots: Vec<u32>,
    pub atlas_samples: Vec<RenderHybridGiScenePrepareSample>,
    pub capture_samples: Vec<RenderHybridGiScenePrepareSample>,
    pub surface_cache_depth_samples: Vec<RenderHybridGiScenePrepareSample>,
    pub surface_cache_pages: Vec<RenderHybridGiSurfaceCachePageRecord>,
    pub voxel_clipmaps: Vec<RenderHybridGiVoxelClipmapRecord>,
    pub voxel_clipmap_ids: Vec<u32>,
    pub voxel_samples: Vec<RenderHybridGiScenePrepareSample>,
    pub voxel_occupancy: Vec<u32>,
    pub voxel_occupancy_masks: Vec<RenderHybridGiVoxelOccupancyMaskRecord>,
    pub voxel_cells: Vec<RenderHybridGiVoxelCellRecord>,
    pub voxel_cell_samples: Vec<RenderHybridGiVoxelCellSampleRecord>,
    pub voxel_cell_dominant_nodes: Vec<RenderHybridGiVoxelCellDominantNodeRecord>,
    pub voxel_cell_dominant_samples: Vec<RenderHybridGiVoxelCellSampleRecord>,
    pub probe_trace_tiles: Vec<RenderHybridGiTraceTileRecord>,
    pub probe_trace_diagnostics: Vec<RenderHybridGiProbeTraceDiagnosticRecord>,
    pub probe_trace_dispatch: [u32; 3],
    pub texture_width: u32,
    pub texture_height: u32,
    pub texture_layers: u32,
}

impl RenderHybridGiScenePrepareReadbackOutputs {
    pub fn has_runtime_feedback_payload(&self) -> bool {
        !self.atlas_samples.is_empty()
            || !self.capture_samples.is_empty()
            || !self.surface_cache_depth_samples.is_empty()
            || !self.surface_cache_pages.is_empty()
            || !self.voxel_clipmaps.is_empty()
            || !self.voxel_samples.is_empty()
            || !self.voxel_occupancy.is_empty()
            || !self.voxel_occupancy_masks.is_empty()
            || !self.voxel_cells.is_empty()
            || !self.voxel_cell_samples.is_empty()
            || !self.voxel_cell_dominant_nodes.is_empty()
            || !self.voxel_cell_dominant_samples.is_empty()
            || !self.probe_trace_tiles.is_empty()
            || !self.probe_trace_diagnostics.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderHybridGiTraceIntersectionSource {
    #[default]
    Miss,
    SurfaceCache,
    GlobalSdf,
    VoxelClipmap,
    HardwareRayTracing,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderHybridGiTraceLightingSource {
    #[default]
    NeutralAmbient,
    SurfaceCache,
    ProbeLineage,
    VoxelRadiance,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderHybridGiTraceFallbackReason {
    #[default]
    None,
    ScreenDataUnavailable,
    HardwareRayTracingUnavailable,
    GlobalSdfUnavailable,
    IntersectionMiss,
    LightingUnavailable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiTraceCostCounters {
    pub texture_samples: u32,
    pub page_tests: u32,
    pub sdf_steps: u32,
    pub voxel_candidates: u32,
    pub hardware_rays: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiProbeTraceDiagnosticRecord {
    pub probe_id: u32,
    pub intersection_source: RenderHybridGiTraceIntersectionSource,
    pub lighting_source: RenderHybridGiTraceLightingSource,
    pub intersection_backend_mask: u32,
    pub lighting_source_mask: u32,
    pub distance_bits: u32,
    pub confidence_bits: u32,
    pub fallback_reason: RenderHybridGiTraceFallbackReason,
    pub cost: RenderHybridGiTraceCostCounters,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiScenePrepareSample {
    pub index: u32,
    pub rgba8: [u8; 4],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiSurfaceCachePageRecord {
    pub page_id: u32,
    pub owner_card_id: u32,
    pub atlas_slot_id: u32,
    pub bounds_center_x_bits: u32,
    pub bounds_center_y_bits: u32,
    pub bounds_center_z_bits: u32,
    pub bounds_radius_bits: u32,
    pub radiance_rgba8: [u8; 4],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiVoxelClipmapRecord {
    pub clipmap_id: u32,
    pub center_x_bits: u32,
    pub center_y_bits: u32,
    pub center_z_bits: u32,
    pub half_extent_bits: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiTraceTileRecord {
    pub tile_id: u32,
    pub probe_id: u32,
    pub trace_region_id: u32,
    pub ray_count: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiVoxelOccupancyMaskRecord {
    pub clipmap_id: u32,
    pub occupancy_mask: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiVoxelCellRecord {
    pub clipmap_id: u32,
    pub cell_id: u32,
    pub occupancy: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiVoxelCellSampleRecord {
    pub clipmap_id: u32,
    pub cell_id: u32,
    pub rgba8: [u8; 4],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiVoxelCellDominantNodeRecord {
    pub clipmap_id: u32,
    pub cell_id: u32,
    pub dominant_node_id: u64,
}
