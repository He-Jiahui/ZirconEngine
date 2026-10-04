use bytemuck::{Pod, Zeroable};

use crate::hybrid_gi::HybridGiPrepareRadianceCacheUpdate;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(in crate::hybrid_gi::renderer) struct GpuRadianceCacheUpdateInput {
    pub(super) slot: u32,
    pub(super) generation_low: u32,
    pub(super) generation_high: u32,
    pub(super) radiance_confidence: u32,
    pub(super) reuse_committed_radiance: u32,
}

impl From<&HybridGiPrepareRadianceCacheUpdate> for GpuRadianceCacheUpdateInput {
    fn from(update: &HybridGiPrepareRadianceCacheUpdate) -> Self {
        Self {
            slot: update.slot,
            generation_low: update.generation as u32,
            generation_high: (update.generation >> 32) as u32,
            radiance_confidence: u32::from_le_bytes([
                update.radiance_rgb[0],
                update.radiance_rgb[1],
                update.radiance_rgb[2],
                update.confidence_q8,
            ]),
            reuse_committed_radiance: u32::from(update.reuse_committed_radiance),
        }
    }
}

#[cfg(test)]
#[path = "tests/gpu_radiance_cache_update_input.rs"]
mod tests;
