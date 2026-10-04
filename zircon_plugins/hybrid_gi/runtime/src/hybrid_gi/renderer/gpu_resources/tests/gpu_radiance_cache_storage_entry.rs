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
