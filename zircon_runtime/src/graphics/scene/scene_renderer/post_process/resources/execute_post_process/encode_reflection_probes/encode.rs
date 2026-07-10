use crate::core::framework::render::RenderFrameExtract;
use crate::core::math::UVec2;
use bytemuck::Zeroable;

use super::super::super::super::constants::MAX_REFLECTION_PROBES;
use super::super::super::super::reflection_probe_gpu::GpuReflectionProbe;

pub(in super::super) fn encode_reflection_probes(
    _extract: &RenderFrameExtract,
    _viewport_size: UVec2,
    _enabled: bool,
) -> ([GpuReflectionProbe; MAX_REFLECTION_PROBES], u32) {
    ([GpuReflectionProbe::zeroed(); MAX_REFLECTION_PROBES], 0)
}
