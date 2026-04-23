use crate::core::math::Vec3;

use super::{
    HybridGiPrepareCardCaptureRequest, HybridGiPrepareVoxelCell, HybridGiPrepareVoxelClipmap,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HybridGiPrepareSurfaceCachePageContent {
    pub(crate) page_id: u32,
    pub(crate) owner_card_id: u32,
    pub(crate) atlas_slot_id: u32,
    pub(crate) capture_slot_id: u32,
    pub(crate) bounds_center: Vec3,
    pub(crate) bounds_radius: f32,
    pub(crate) atlas_sample_rgba: [u8; 4],
    pub(crate) capture_sample_rgba: [u8; 4],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HybridGiScenePrepareFrame {
    pub(crate) card_capture_requests: Vec<HybridGiPrepareCardCaptureRequest>,
    pub(crate) surface_cache_page_contents: Vec<HybridGiPrepareSurfaceCachePageContent>,
    pub(crate) voxel_clipmaps: Vec<HybridGiPrepareVoxelClipmap>,
    pub(crate) voxel_cells: Vec<HybridGiPrepareVoxelCell>,
}
