use crate::core::framework::render::RenderFrameExtract;
use crate::core::math::UVec2;
use bytemuck::Zeroable;

use super::super::super::super::constants::MAX_REFLECTION_PROBES;
use super::super::super::super::reflection_probe_gpu::GpuReflectionProbe;

/// 后处理的旧屏幕空间反射探针通道保持中性；当前环境探针由场景光照自己的资源链提供。
/// 即使特性启用，此通道也返回零个有效条目，上传层据计数跳过旧缓冲写入。
pub(in super::super) fn encode_reflection_probes(
    _extract: &RenderFrameExtract,
    _viewport_size: UVec2,
    _enabled: bool,
) -> ([GpuReflectionProbe; MAX_REFLECTION_PROBES], u32) {
    ([GpuReflectionProbe::zeroed(); MAX_REFLECTION_PROBES], 0)
}
