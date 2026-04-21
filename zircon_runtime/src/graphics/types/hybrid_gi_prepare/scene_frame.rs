use super::{
    HybridGiPrepareCardCaptureRequest, HybridGiPrepareVoxelCell, HybridGiPrepareVoxelClipmap,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HybridGiScenePrepareFrame {
    pub(crate) card_capture_requests: Vec<HybridGiPrepareCardCaptureRequest>,
    pub(crate) voxel_clipmaps: Vec<HybridGiPrepareVoxelClipmap>,
    pub(crate) voxel_cells: Vec<HybridGiPrepareVoxelCell>,
}
