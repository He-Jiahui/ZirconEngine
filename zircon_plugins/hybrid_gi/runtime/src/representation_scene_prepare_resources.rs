use crate::hybrid_gi::HybridGiPrepareVoxelCell;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiRuntimeScenePrepareResources {
    atlas_slot_rgba_samples: Vec<(u32, [u8; 4])>,
    capture_slot_rgba_samples: Vec<(u32, [u8; 4])>,
    voxel_cells: Vec<HybridGiPrepareVoxelCell>,
}

pub(crate) trait HybridGiScenePrepareResourceSamples {
    fn atlas_slot_rgba_sample(&self, atlas_slot_id: u32) -> Option<[u8; 4]>;

    fn capture_slot_rgba_sample(&self, capture_slot_id: u32) -> Option<[u8; 4]>;

    fn voxel_cells(&self) -> &[HybridGiPrepareVoxelCell];
}

impl HybridGiRuntimeScenePrepareResources {
    pub(crate) fn new(
        atlas_slot_rgba_samples: Vec<(u32, [u8; 4])>,
        capture_slot_rgba_samples: Vec<(u32, [u8; 4])>,
    ) -> Self {
        Self {
            atlas_slot_rgba_samples,
            capture_slot_rgba_samples,
            voxel_cells: Vec::new(),
        }
    }

    pub(crate) fn with_voxel_cells(mut self, voxel_cells: Vec<HybridGiPrepareVoxelCell>) -> Self {
        self.voxel_cells = voxel_cells;
        self
    }
}

impl HybridGiScenePrepareResourceSamples for HybridGiRuntimeScenePrepareResources {
    fn atlas_slot_rgba_sample(&self, atlas_slot_id: u32) -> Option<[u8; 4]> {
        self.atlas_slot_rgba_samples
            .iter()
            .find_map(|(slot_id, rgba)| (*slot_id == atlas_slot_id).then_some(*rgba))
    }

    fn capture_slot_rgba_sample(&self, capture_slot_id: u32) -> Option<[u8; 4]> {
        self.capture_slot_rgba_samples
            .iter()
            .find_map(|(slot_id, rgba)| (*slot_id == capture_slot_id).then_some(*rgba))
    }

    fn voxel_cells(&self) -> &[HybridGiPrepareVoxelCell] {
        &self.voxel_cells
    }
}
