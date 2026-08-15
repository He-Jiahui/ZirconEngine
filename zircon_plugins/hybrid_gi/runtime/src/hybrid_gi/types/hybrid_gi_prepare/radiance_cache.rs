pub const HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT: usize = 8;
pub const HYBRID_GI_RADIANCE_CACHE_MAX_RESIDENT_PROBE_COUNT: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HybridGiPrepareRadianceCacheUpdate {
    pub slot: u32,
    pub generation: u64,
    pub radiance_rgb: [u8; 3],
    pub confidence_q8: u8,
    pub reuse_committed_radiance: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HybridGiPrepareRadianceCacheConsume {
    pub probe_id: u32,
    pub generation: u64,
    pub slots: [u32; HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT],
    pub weights_q16: [u16; HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT],
}
