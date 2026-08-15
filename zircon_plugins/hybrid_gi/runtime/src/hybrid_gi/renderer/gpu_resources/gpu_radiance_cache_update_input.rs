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
mod tests {
    use super::*;

    #[test]
    fn gpu_radiance_cache_update_input_preserves_generation_and_sample_bits() {
        let input = GpuRadianceCacheUpdateInput::from(&HybridGiPrepareRadianceCacheUpdate {
            slot: 7,
            generation: 0x0123_4567_89ab_cdef,
            radiance_rgb: [10, 20, 30],
            confidence_q8: 40,
            reuse_committed_radiance: true,
        });

        assert_eq!(input.slot, 7);
        assert_eq!(input.generation_low, 0x89ab_cdef);
        assert_eq!(input.generation_high, 0x0123_4567);
        assert_eq!(input.radiance_confidence.to_le_bytes(), [10, 20, 30, 40]);
        assert_eq!(input.reuse_committed_radiance, 1);
    }
}
