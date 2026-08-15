use zircon_runtime::core::math::Vec3;

use super::{
    HybridGiPrepareCardCaptureRequest, HybridGiPrepareRadianceCacheConsume,
    HybridGiPrepareRadianceCacheUpdate, HybridGiPrepareVoxelCell, HybridGiPrepareVoxelClipmap,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HybridGiPrepareSurfaceCachePageContent {
    pub page_id: u32,
    pub owner_card_id: u32,
    pub atlas_slot_id: u32,
    pub capture_slot_id: u32,
    pub bounds_center: Vec3,
    pub bounds_radius: f32,
    pub atlas_sample_rgba: [u8; 4],
    pub capture_sample_rgba: [u8; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HybridGiPrepareSurfaceCacheDepthSourceSample {
    pub page_id: u32,
    pub atlas_slot_id: u32,
    pub depth_rgba: [u8; 4],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HybridGiScenePrepareFrame {
    pub card_capture_requests: Vec<HybridGiPrepareCardCaptureRequest>,
    pub surface_cache_page_contents: Vec<HybridGiPrepareSurfaceCachePageContent>,
    pub voxel_clipmaps: Vec<HybridGiPrepareVoxelClipmap>,
    pub voxel_cells: Vec<HybridGiPrepareVoxelCell>,
    /// Presentation-local card IDs resolve to these stable prepared-geometry identities.
    pub card_owner_stable_instance_keys: Vec<(u32, u64)>,
    /// Full committed snapshot used only when a renderer instance must rebuild evicted GPU state.
    pub radiance_cache_bootstrap_updates: Vec<HybridGiPrepareRadianceCacheUpdate>,
    pub radiance_cache_updates: Vec<HybridGiPrepareRadianceCacheUpdate>,
    pub radiance_cache_consumes: Vec<HybridGiPrepareRadianceCacheConsume>,
}
