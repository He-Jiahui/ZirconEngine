pub const HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT: usize = 8;
pub const HYBRID_GI_RADIANCE_CACHE_MAX_RESIDENT_PROBE_COUNT: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 单槽辐射缓存上传；复用标志用于滚动后沿用已提交辐射，启动快照始终要求重新写入。
pub struct HybridGiPrepareRadianceCacheUpdate {
    pub slot: u32,
    pub generation: u64,
    pub radiance_rgb: [u8; 3],
    pub confidence_q8: u8,
    pub reuse_committed_radiance: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// 屏幕探针的八角插值映射；槽位与 Q16 权重只引用同代已可见的驻留探针。
pub struct HybridGiPrepareRadianceCacheConsume {
    pub probe_id: u32,
    pub generation: u64,
    pub slots: [u32; HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT],
    pub weights_q16: [u16; HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT],
}
