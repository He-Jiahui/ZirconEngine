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
#[path = "tests/gpu_radiance_cache_consume_input.rs"]
mod tests;
