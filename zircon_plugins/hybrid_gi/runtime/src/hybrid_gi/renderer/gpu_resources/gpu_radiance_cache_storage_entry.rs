use bytemuck::{Pod, Zeroable};

// Buffer-backed RC atlas ABI shared by allocation, WGSL indexing, and readback tests.
pub(super) const GPU_RADIANCE_CACHE_PROBE_TILE_EXTENT: usize = 4;
pub(super) const GPU_RADIANCE_CACHE_PROBE_BASE_TILE_WORD_COUNT: usize =
    GPU_RADIANCE_CACHE_PROBE_TILE_EXTENT * GPU_RADIANCE_CACHE_PROBE_TILE_EXTENT;
pub(super) const GPU_RADIANCE_CACHE_PROBE_MIP1_WORD_COUNT: usize = 4;
pub(super) const GPU_RADIANCE_CACHE_PROBE_MIP2_WORD_COUNT: usize = 1;
pub(super) const GPU_RADIANCE_CACHE_PROBE_MIP1_WORD_OFFSET: usize =
    GPU_RADIANCE_CACHE_PROBE_BASE_TILE_WORD_COUNT;
pub(super) const GPU_RADIANCE_CACHE_PROBE_MIP2_WORD_OFFSET: usize =
    GPU_RADIANCE_CACHE_PROBE_MIP1_WORD_OFFSET + GPU_RADIANCE_CACHE_PROBE_MIP1_WORD_COUNT;
pub(super) const GPU_RADIANCE_CACHE_PROBE_MIP_WORD_COUNT: usize =
    GPU_RADIANCE_CACHE_PROBE_MIP1_WORD_COUNT + GPU_RADIANCE_CACHE_PROBE_MIP2_WORD_COUNT;
pub(super) const GPU_RADIANCE_CACHE_PROBE_ATLAS_WORD_COUNT: usize =
    GPU_RADIANCE_CACHE_PROBE_MIP2_WORD_OFFSET + GPU_RADIANCE_CACHE_PROBE_MIP2_WORD_COUNT;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(in crate::hybrid_gi::renderer) struct GpuRadianceCacheStorageEntry {
    pub(super) radiance_confidence: u32,
    pub(super) generation_low: u32,
    pub(super) generation_high: u32,
    pub(super) atlas_base: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radiance_cache_probe_atlas_layout_reserves_base_border_and_mips() {
        assert_eq!(GPU_RADIANCE_CACHE_PROBE_TILE_EXTENT, 4);
        assert_eq!(GPU_RADIANCE_CACHE_PROBE_BASE_TILE_WORD_COUNT, 16);
        assert_eq!(GPU_RADIANCE_CACHE_PROBE_MIP1_WORD_OFFSET, 16);
        assert_eq!(GPU_RADIANCE_CACHE_PROBE_MIP2_WORD_OFFSET, 20);
        assert_eq!(GPU_RADIANCE_CACHE_PROBE_MIP_WORD_COUNT, 5);
        assert_eq!(GPU_RADIANCE_CACHE_PROBE_ATLAS_WORD_COUNT, 21);
        assert_eq!(
            std::mem::size_of::<GpuRadianceCacheStorageEntry>(),
            4 * std::mem::size_of::<u32>()
        );
    }
}
