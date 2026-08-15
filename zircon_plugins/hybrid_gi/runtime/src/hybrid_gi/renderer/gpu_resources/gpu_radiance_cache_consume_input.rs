use bytemuck::{Pod, Zeroable};

use crate::hybrid_gi::{
    HybridGiPrepareRadianceCacheConsume, HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(in crate::hybrid_gi::renderer) struct GpuRadianceCacheConsumeInput {
    pub(super) probe_id: u32,
    pub(super) generation_low: u32,
    pub(super) generation_high: u32,
    pub(super) resident_probe_index: u32,
    pub(super) slots: [u32; HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT],
    pub(super) weights_q16: [u32; HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT],
}

impl GpuRadianceCacheConsumeInput {
    pub(in crate::hybrid_gi::renderer::gpu_resources) fn new(
        consume: &HybridGiPrepareRadianceCacheConsume,
        resident_probe_index: u32,
    ) -> Self {
        Self {
            probe_id: consume.probe_id,
            generation_low: consume.generation as u32,
            generation_high: (consume.generation >> 32) as u32,
            resident_probe_index,
            slots: consume.slots,
            weights_q16: consume.weights_q16.map(u32::from),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_radiance_cache_consume_input_widens_all_eight_corner_weights() {
        let input = GpuRadianceCacheConsumeInput::new(
            &HybridGiPrepareRadianceCacheConsume {
                probe_id: 11,
                generation: 0xfedc_ba98_7654_3210,
                slots: [1, 2, 3, 4, 5, 6, 7, 8],
                weights_q16: [0, 1, 2, 3, 4, 5, 6, u16::MAX],
            },
            13,
        );

        assert_eq!(input.probe_id, 11);
        assert_eq!(input.generation_low, 0x7654_3210);
        assert_eq!(input.generation_high, 0xfedc_ba98);
        assert_eq!(input.resident_probe_index, 13);
        assert_eq!(input.slots, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(
            input.weights_q16,
            [0, 1, 2, 3, 4, 5, 6, u32::from(u16::MAX)]
        );
    }
}
