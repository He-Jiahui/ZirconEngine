use crate::core::framework::render::{ProceduralSkyParams, STANDARD_MATERIAL_MIN_ROUGHNESS};
use crate::core::math::{Real, Vec3};

/// CPU 端渐变端点基准，目前由本模块测试使用；完整天空与太阳由共享 WGSL 模型计算。
pub(crate) fn procedural_sky_color_at_vertical(
    params: ProceduralSkyParams,
    vertical01: Real,
) -> Vec3 {
    let t = vertical01.clamp(0.0, 1.0);
    params
        .horizon_color
        .truncate()
        .lerp(params.zenith_color.truncate(), t)
        * params.intensity.max(0.0)
}

/// 将 smoothness 契约转换为可用于 PBR 采样的粗糙度，镜面端保留材质约定的最小值。
pub(crate) fn roughness_from_smoothness(smoothness: Real) -> Real {
    (1.0 - smoothness).clamp(STANDARD_MATERIAL_MIN_ROUGHNESS, 1.0)
}

#[cfg(test)]
#[path = "tests/procedural_environment.rs"]
mod tests;
