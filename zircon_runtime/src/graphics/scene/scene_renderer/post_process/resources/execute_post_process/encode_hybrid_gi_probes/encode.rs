use crate::core::math::UVec2;
use bytemuck::Zeroable;

use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::constants::MAX_HYBRID_GI_PROBES;
use super::super::super::super::hybrid_gi_probe_gpu::GpuHybridGiProbe;

pub(in super::super) fn encode_hybrid_gi_probes(
    _frame: &ViewportRenderFrame,
    _viewport_size: UVec2,
    _enabled: bool,
) -> ([GpuHybridGiProbe; MAX_HYBRID_GI_PROBES], u32, u32) {
    ([GpuHybridGiProbe::zeroed(); MAX_HYBRID_GI_PROBES], 0, 0)
}
