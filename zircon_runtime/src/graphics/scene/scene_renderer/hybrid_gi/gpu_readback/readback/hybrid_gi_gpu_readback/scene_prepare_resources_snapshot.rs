#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiScenePrepareResourcesSnapshot {
    pub(super) card_capture_request_count: u32,
    pub(super) voxel_clipmap_ids: Vec<u32>,
    pub(super) occupied_atlas_slots: Vec<u32>,
    pub(super) occupied_capture_slots: Vec<u32>,
    pub(super) atlas_slot_rgba_samples: Vec<(u32, [u8; 4])>,
    pub(super) capture_slot_rgba_samples: Vec<(u32, [u8; 4])>,
    pub(super) voxel_clipmap_rgba_samples: Vec<(u32, [u8; 4])>,
    pub(super) voxel_clipmap_occupancy_masks: Vec<(u32, u64)>,
    pub(super) voxel_clipmap_cell_rgba_samples: Vec<(u32, u32, [u8; 4])>,
    pub(super) voxel_clipmap_cell_occupancy_counts: Vec<(u32, u32, u32)>,
    pub(super) voxel_clipmap_cell_dominant_node_ids: Vec<(u32, u32, u64)>,
    pub(super) voxel_clipmap_cell_dominant_rgba_samples: Vec<(u32, u32, [u8; 4])>,
    pub(super) atlas_slot_count: u32,
    pub(super) capture_slot_count: u32,
    pub(super) atlas_texture_extent: (u32, u32),
    pub(super) capture_texture_extent: (u32, u32),
    pub(super) capture_layer_count: u32,
}

impl HybridGiScenePrepareResourcesSnapshot {
    pub(crate) fn new(
        card_capture_request_count: u32,
        voxel_clipmap_ids: Vec<u32>,
        occupied_atlas_slots: Vec<u32>,
        occupied_capture_slots: Vec<u32>,
        atlas_slot_count: u32,
        capture_slot_count: u32,
        atlas_texture_extent: (u32, u32),
        capture_texture_extent: (u32, u32),
        capture_layer_count: u32,
    ) -> Self {
        Self {
            card_capture_request_count,
            voxel_clipmap_ids,
            occupied_atlas_slots,
            occupied_capture_slots,
            atlas_slot_rgba_samples: Vec::new(),
            capture_slot_rgba_samples: Vec::new(),
            voxel_clipmap_rgba_samples: Vec::new(),
            voxel_clipmap_occupancy_masks: Vec::new(),
            voxel_clipmap_cell_rgba_samples: Vec::new(),
            voxel_clipmap_cell_occupancy_counts: Vec::new(),
            voxel_clipmap_cell_dominant_node_ids: Vec::new(),
            voxel_clipmap_cell_dominant_rgba_samples: Vec::new(),
            atlas_slot_count,
            capture_slot_count,
            atlas_texture_extent,
            capture_texture_extent,
            capture_layer_count,
        }
    }
}
