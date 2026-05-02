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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderVirtualGeometryNodeClusterCullReadbackOutputs {
    pub traversal_records: Vec<RenderVirtualGeometryNodeAndClusterCullTraversalRecord>,
    pub child_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullChildWorkItem>,
    pub cluster_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullClusterWorkItem>,
    pub launch_worklist_snapshots:
        Vec<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>,
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
    pub voxel_cells: Vec<RenderHybridGiVoxelCellRecord>,
    pub texture_width: u32,
    pub texture_height: u32,
    pub texture_layers: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiScenePrepareSample {
    pub index: u32,
    pub rgba8: [u8; 4],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiVoxelCellRecord {
    pub clipmap_id: u32,
    pub cell_id: u32,
    pub occupancy: u32,
}

#[cfg(test)]
mod tests {
    use super::RenderPluginRendererOutputs;

    #[test]
    fn default_plugin_renderer_outputs_are_empty() {
        let outputs = RenderPluginRendererOutputs::default();

        assert!(outputs.virtual_geometry.page_table_entries.is_empty());
        assert!(outputs.virtual_geometry.selected_clusters.is_empty());
        assert!(outputs.hybrid_gi.completed_probe_ids.is_empty());
        assert!(outputs.hybrid_gi.scene_prepare.voxel_cells.is_empty());
    }
}
