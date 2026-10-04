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
#[path = "tests/gpu_radiance_cache_storage_entry.rs"]
mod tests;
