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
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderVirtualGeometryNodeClusterCullReadbackOutputs {
    pub traversal_records: Vec<RenderVirtualGeometryNodeAndClusterCullTraversalRecord>,
    pub child_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullChildWorkItem>,
    pub cluster_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullClusterWorkItem>,
    pub launch_worklist_snapshots:
        Vec<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>,
}

impl RenderVirtualGeometryNodeClusterCullReadbackOutputs {
    pub fn is_empty(&self) -> bool {
        self.traversal_records.is_empty()
            && self.child_work_items.is_empty()
            && self.cluster_work_items.is_empty()
            && self.launch_worklist_snapshots.is_empty()
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiReadbackOutputs {
    pub cache_entries: Vec<RenderHybridGiCacheEntryRecord>,
    pub completed_probe_ids: Vec<u32>,
    pub completed_trace_region_ids: Vec<u32>,
    pub probe_irradiance_rgb: Vec<[u16; 3]>,
    pub probe_rt_lighting_rgb: Vec<[u16; 3]>,
    pub scene_prepare: RenderHybridGiScenePrepareReadbackOutputs,
}

impl RenderHybridGiReadbackOutputs {
    pub fn is_empty(&self) -> bool {
        self.cache_entries.is_empty()
            && self.completed_probe_ids.is_empty()
            && self.completed_trace_region_ids.is_empty()
            && self.probe_irradiance_rgb.is_empty()
            && self.probe_rt_lighting_rgb.is_empty()
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
    pub voxel_clipmap_ids: Vec<u32>,
    pub voxel_samples: Vec<RenderHybridGiScenePrepareSample>,
    pub voxel_occupancy: Vec<u32>,
    pub voxel_occupancy_masks: Vec<RenderHybridGiVoxelOccupancyMaskRecord>,
    pub voxel_cells: Vec<RenderHybridGiVoxelCellRecord>,
    pub voxel_cell_samples: Vec<RenderHybridGiVoxelCellSampleRecord>,
    pub voxel_cell_dominant_nodes: Vec<RenderHybridGiVoxelCellDominantNodeRecord>,
    pub voxel_cell_dominant_samples: Vec<RenderHybridGiVoxelCellSampleRecord>,
    pub texture_width: u32,
    pub texture_height: u32,
    pub texture_layers: u32,
}

impl RenderHybridGiScenePrepareReadbackOutputs {
    pub fn has_runtime_feedback_payload(&self) -> bool {
        !self.atlas_samples.is_empty()
            || !self.capture_samples.is_empty()
            || !self.voxel_samples.is_empty()
            || !self.voxel_occupancy.is_empty()
            || !self.voxel_occupancy_masks.is_empty()
            || !self.voxel_cells.is_empty()
            || !self.voxel_cell_samples.is_empty()
            || !self.voxel_cell_dominant_nodes.is_empty()
            || !self.voxel_cell_dominant_samples.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiScenePrepareSample {
    pub index: u32,
    pub rgba8: [u8; 4],
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
        RenderHybridGiReadbackOutputs, RenderHybridGiScenePrepareReadbackOutputs,
        RenderHybridGiScenePrepareSample, RenderParticleGpuReadbackOutputs,
        RenderPluginRendererOutputs, RenderVirtualGeometryExecutionState,
        RenderVirtualGeometryNodeClusterCullReadbackOutputs, RenderVirtualGeometryReadbackOutputs,
        RenderVirtualGeometryVisBuffer64Entry,
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
                traversal_records: vec![Default::default()],
                ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
            },
            ..RenderVirtualGeometryReadbackOutputs::default()
        };

        assert!(!outputs.node_cluster_cull.is_empty());
        assert!(!outputs.is_empty());

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
}
