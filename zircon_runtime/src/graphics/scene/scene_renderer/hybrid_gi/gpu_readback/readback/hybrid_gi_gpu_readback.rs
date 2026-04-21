#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiScenePrepareResourcesSnapshot {
    pub(crate) card_capture_request_count: u32,
    pub(crate) voxel_clipmap_ids: Vec<u32>,
    pub(crate) occupied_atlas_slots: Vec<u32>,
    pub(crate) occupied_capture_slots: Vec<u32>,
    pub(crate) atlas_slot_rgba_samples: Vec<(u32, [u8; 4])>,
    pub(crate) capture_slot_rgba_samples: Vec<(u32, [u8; 4])>,
    pub(crate) voxel_clipmap_rgba_samples: Vec<(u32, [u8; 4])>,
    pub(crate) voxel_clipmap_occupancy_masks: Vec<(u32, u64)>,
    pub(crate) voxel_clipmap_cell_rgba_samples: Vec<(u32, u32, [u8; 4])>,
    pub(crate) voxel_clipmap_cell_occupancy_counts: Vec<(u32, u32, u32)>,
    pub(crate) voxel_clipmap_cell_dominant_node_ids: Vec<(u32, u32, u64)>,
    pub(crate) voxel_clipmap_cell_dominant_rgba_samples: Vec<(u32, u32, [u8; 4])>,
    pub(crate) atlas_slot_count: u32,
    pub(crate) capture_slot_count: u32,
    pub(crate) atlas_texture_extent: (u32, u32),
    pub(crate) capture_texture_extent: (u32, u32),
    pub(crate) capture_layer_count: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiGpuReadback {
    pub(crate) cache_entries: Vec<(u32, u32)>,
    pub(crate) completed_probe_ids: Vec<u32>,
    pub(crate) completed_trace_region_ids: Vec<u32>,
    pub(crate) probe_irradiance_rgb: Vec<(u32, [u8; 3])>,
    pub(crate) probe_trace_lighting_rgb: Vec<(u32, [u8; 3])>,
    pub(crate) scene_prepare_resources: Option<HybridGiScenePrepareResourcesSnapshot>,
}
