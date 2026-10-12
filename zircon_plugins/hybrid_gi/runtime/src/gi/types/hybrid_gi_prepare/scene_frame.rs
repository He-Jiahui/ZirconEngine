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
/// 场景准备输出：脏页请求与持久页内容并列保留，GPU 可按槽覆盖采集深度源并恢复被驱逐的缓存实例。
pub struct HybridGiScenePrepareFrame {
    pub card_capture_requests: Vec<HybridGiPrepareCardCaptureRequest>,
    pub surface_cache_page_contents: Vec<HybridGiPrepareSurfaceCachePageContent>,
    pub voxel_clipmaps: Vec<HybridGiPrepareVoxelClipmap>,
    pub voxel_cells: Vec<HybridGiPrepareVoxelCell>,
    /// 将本次呈现的卡 ID 映射到已准备几何的稳定实例键，供重建页内容时查找所有者。
    pub card_owner_stable_instance_keys: Vec<(u32, u64)>,
    /// 完整已提交快照，仅供新建或被驱逐的渲染实例重建 GPU 辐射缓存；常规帧使用增量更新。
    pub radiance_cache_bootstrap_updates: Vec<HybridGiPrepareRadianceCacheUpdate>,
    pub radiance_cache_updates: Vec<HybridGiPrepareRadianceCacheUpdate>,
    pub radiance_cache_consumes: Vec<HybridGiPrepareRadianceCacheConsume>,
}
