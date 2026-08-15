use super::{
    RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryNodeAndClusterCullChildWorkItem,
    RenderVirtualGeometryNodeAndClusterCullClusterWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
    RenderVirtualGeometryNodeAndClusterCullTraversalRecord, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometryVisBuffer64Entry,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderPluginRendererOutputs {
    pub virtual_geometry: RenderVirtualGeometryReadbackOutputs,
    pub hybrid_gi: RenderHybridGiReadbackOutputs,
    pub particles: RenderParticleGpuReadbackOutputs,
}

impl RenderPluginRendererOutputs {
    pub fn is_empty(&self) -> bool {
        self.virtual_geometry.is_empty() && self.hybrid_gi.is_empty() && self.particles.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderParticleGpuReadbackOutputs {
    pub alive_count: u32,
    pub spawned_total: u32,
    pub debug_flags: u32,
    pub per_emitter_spawned: Vec<u32>,
    pub indirect_draw_args: [u32; 4],
}

impl RenderParticleGpuReadbackOutputs {
    pub fn is_empty(&self) -> bool {
        self.alive_count == 0
            && self.spawned_total == 0
            && self.debug_flags == 0
            && self.per_emitter_spawned.is_empty()
            && self.indirect_draw_args == [0; 4]
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderVirtualGeometryReadbackOutputs {
    pub page_table_entries: Vec<u32>,
    pub completed_page_assignments: Vec<RenderVirtualGeometryPageAssignmentRecord>,
    pub page_replacements: Vec<RenderVirtualGeometryPageReplacementRecord>,
    pub selected_clusters: Vec<RenderVirtualGeometrySelectedCluster>,
    pub visbuffer64_entries: Vec<RenderVirtualGeometryVisBuffer64Entry>,
    pub hardware_rasterization_records: Vec<RenderVirtualGeometryHardwareRasterizationRecord>,
    pub node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs,
}

impl RenderVirtualGeometryReadbackOutputs {
    pub fn is_empty(&self) -> bool {
        self.page_table_entries.is_empty()
            && self.completed_page_assignments.is_empty()
            && self.page_replacements.is_empty()
            && self.selected_clusters.is_empty()
            && self.visbuffer64_entries.is_empty()
            && self.hardware_rasterization_records.is_empty()
            && self.node_cluster_cull.is_empty()
    }

    pub fn take_node_and_cluster_cull_page_request_ids(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.node_cluster_cull.page_request_ids)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderVirtualGeometryNodeClusterCullReadbackOutputs {
    pub traversal_records: Vec<RenderVirtualGeometryNodeAndClusterCullTraversalRecord>,
    pub child_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullChildWorkItem>,
    pub cluster_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullClusterWorkItem>,
    pub launch_worklist_snapshots:
        Vec<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>,
    pub page_request_ids: Vec<u32>,
}

impl RenderVirtualGeometryNodeClusterCullReadbackOutputs {
    pub fn is_empty(&self) -> bool {
        self.traversal_records.is_empty()
            && self.child_work_items.is_empty()
            && self.cluster_work_items.is_empty()
            && self.launch_worklist_snapshots.is_empty()
            && self.page_request_ids.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryPageAssignmentRecord {
    pub page_id: u64,
    pub physical_slot: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryPageReplacementRecord {
    pub old_page_id: u64,
    pub new_page_id: u64,
    pub physical_slot: u32,
}

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

#[cfg(test)]
mod tests {
    use super::{
        RenderHybridGiProbeTraceDiagnosticRecord, RenderHybridGiReadbackOutputs,
        RenderHybridGiScenePrepareReadbackOutputs, RenderHybridGiScenePrepareSample,
        RenderHybridGiSurfaceCachePageRecord, RenderHybridGiTraceTileRecord,
        RenderHybridGiVoxelClipmapRecord, RenderParticleGpuReadbackOutputs,
        RenderPluginRendererOutputs, RenderVirtualGeometryNodeAndClusterCullTraversalRecord,
        RenderVirtualGeometryNodeClusterCullReadbackOutputs, RenderVirtualGeometryReadbackOutputs,
        RenderVirtualGeometryVisBuffer64Entry,
    };
    use crate::core::framework::render::{
        RenderVirtualGeometryExecutionState,
        RenderVirtualGeometryNodeAndClusterCullTraversalChildSource,
        RenderVirtualGeometryNodeAndClusterCullTraversalOp,
    };

    #[test]
    fn default_plugin_renderer_outputs_are_empty() {
        let outputs = RenderPluginRendererOutputs::default();

        assert!(outputs.virtual_geometry.page_table_entries.is_empty());
        assert!(outputs.virtual_geometry.selected_clusters.is_empty());
        assert!(outputs.hybrid_gi.completed_probe_ids.is_empty());
        assert!(outputs.hybrid_gi.scene_prepare.voxel_cells.is_empty());
        assert_eq!(outputs.particles.alive_count, 0);
        assert!(outputs.particles.per_emitter_spawned.is_empty());
        assert!(outputs.is_empty());
        assert!(outputs.virtual_geometry.is_empty());
        assert!(outputs.hybrid_gi.is_empty());
    }

    #[test]
    fn particle_gpu_readback_outputs_are_empty_only_without_payloads() {
        let empty = RenderParticleGpuReadbackOutputs::default();
        assert!(empty.is_empty());

        let with_alive_count = RenderParticleGpuReadbackOutputs {
            alive_count: 4,
            ..RenderParticleGpuReadbackOutputs::default()
        };
        assert!(!with_alive_count.is_empty());

        let with_indirect_args = RenderParticleGpuReadbackOutputs {
            indirect_draw_args: [6, 4, 0, 0],
            ..RenderParticleGpuReadbackOutputs::default()
        };
        assert!(!with_indirect_args.is_empty());
    }

    #[test]
    fn virtual_geometry_readback_outputs_report_node_cluster_cull_payloads() {
        let outputs = RenderVirtualGeometryReadbackOutputs {
            node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                traversal_records: vec![RenderVirtualGeometryNodeAndClusterCullTraversalRecord {
                    op: RenderVirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
                    child_source: RenderVirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                    instance_index: 0,
                    entity: 0,
                    cluster_array_index: 0,
                    hierarchy_node_id: None,
                    node_cluster_start: 0,
                    node_cluster_count: 0,
                    child_base: 0,
                    child_count: 0,
                    traversal_index: 0,
                    cluster_budget: 0,
                    page_budget: 0,
                    forced_mip: None,
                }],
                page_request_ids: vec![300, 301],
                ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
            },
            ..RenderVirtualGeometryReadbackOutputs::default()
        };

        assert!(!outputs.node_cluster_cull.is_empty());
        assert!(!outputs.is_empty());

        let mut outputs = outputs;
        assert_eq!(
            outputs.take_node_and_cluster_cull_page_request_ids(),
            vec![300, 301]
        );
        assert!(outputs.node_cluster_cull.page_request_ids.is_empty());

        let outputs = RenderVirtualGeometryReadbackOutputs {
            visbuffer64_entries: vec![RenderVirtualGeometryVisBuffer64Entry {
                entry_index: 0,
                packed_value: RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
                instance_index: None,
                entity: 0,
                cluster_id: 0,
                page_id: 0,
                lod_level: 0,
                state: RenderVirtualGeometryExecutionState::Missing,
            }],
            ..RenderVirtualGeometryReadbackOutputs::default()
        };

        assert!(!outputs.is_empty());
    }

    #[test]
    fn hybrid_gi_readback_outputs_ignore_non_runtime_scene_prepare_metadata_for_feedback() {
        let outputs = RenderHybridGiReadbackOutputs {
            scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
                occupied_atlas_slots: vec![1],
                texture_width: 64,
                texture_height: 64,
                texture_layers: 2,
                ..RenderHybridGiScenePrepareReadbackOutputs::default()
            },
            ..RenderHybridGiReadbackOutputs::default()
        };

        assert!(!outputs.scene_prepare.has_runtime_feedback_payload());
        assert!(outputs.is_empty());
    }

    #[test]
    fn hybrid_gi_global_sdf_stats_are_runtime_feedback_even_when_zeroed() {
        let outputs = RenderHybridGiReadbackOutputs {
            global_sdf_stats: Some(Default::default()),
            ..RenderHybridGiReadbackOutputs::default()
        };

        assert!(!outputs.is_empty());
    }

    #[test]
    fn hybrid_gi_readback_outputs_report_scene_prepare_runtime_payloads() {
        let outputs = RenderHybridGiReadbackOutputs {
            scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
                voxel_samples: vec![RenderHybridGiScenePrepareSample {
                    index: 4,
                    rgba8: [8, 16, 24, 255],
                }],
                ..RenderHybridGiScenePrepareReadbackOutputs::default()
            },
            ..RenderHybridGiReadbackOutputs::default()
        };

        assert!(outputs.scene_prepare.has_runtime_feedback_payload());
        assert!(!outputs.is_empty());
    }

    #[test]
    fn hybrid_gi_readback_outputs_report_world_space_trace_lookup_records() {
        let outputs = RenderHybridGiReadbackOutputs {
            scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
                surface_cache_pages: vec![RenderHybridGiSurfaceCachePageRecord {
                    page_id: 3,
                    owner_card_id: 7,
                    atlas_slot_id: 11,
                    bounds_center_x_bits: 1.0_f32.to_bits(),
                    bounds_center_y_bits: 2.0_f32.to_bits(),
                    bounds_center_z_bits: 3.0_f32.to_bits(),
                    bounds_radius_bits: 4.0_f32.to_bits(),
                    radiance_rgba8: [24, 48, 96, 255],
                }],
                voxel_clipmaps: vec![RenderHybridGiVoxelClipmapRecord {
                    clipmap_id: 5,
                    center_x_bits: 0.0_f32.to_bits(),
                    center_y_bits: 1.0_f32.to_bits(),
                    center_z_bits: 2.0_f32.to_bits(),
                    half_extent_bits: 8.0_f32.to_bits(),
                }],
                ..RenderHybridGiScenePrepareReadbackOutputs::default()
            },
            ..RenderHybridGiReadbackOutputs::default()
        };

        assert!(outputs.scene_prepare.has_runtime_feedback_payload());
        assert!(!outputs.is_empty());
    }

    #[test]
    fn hybrid_gi_readback_outputs_report_depth_and_trace_tile_payloads() {
        let outputs = RenderHybridGiReadbackOutputs {
            scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
                surface_cache_depth_samples: vec![RenderHybridGiScenePrepareSample {
                    index: 2,
                    rgba8: [96, 96, 96, 255],
                }],
                probe_trace_tiles: vec![RenderHybridGiTraceTileRecord {
                    tile_id: 0,
                    probe_id: 4,
                    trace_region_id: 9,
                    ray_count: 32,
                }],
                probe_trace_dispatch: [1, 1, 1],
                ..RenderHybridGiScenePrepareReadbackOutputs::default()
            },
            ..RenderHybridGiReadbackOutputs::default()
        };

        assert!(outputs.scene_prepare.has_runtime_feedback_payload());
        assert!(!outputs.is_empty());
    }

    #[test]
    fn hybrid_gi_readback_outputs_report_probe_trace_diagnostics() {
        let outputs = RenderHybridGiReadbackOutputs {
            scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
                probe_trace_diagnostics: vec![RenderHybridGiProbeTraceDiagnosticRecord {
                    probe_id: 7,
                    ..RenderHybridGiProbeTraceDiagnosticRecord::default()
                }],
                ..RenderHybridGiScenePrepareReadbackOutputs::default()
            },
            ..RenderHybridGiReadbackOutputs::default()
        };

        assert!(outputs.scene_prepare.has_runtime_feedback_payload());
        assert!(!outputs.is_empty());
    }
}
