use crate::core::math::Real;

use super::environment_pbr_recipe::{
    ENVIRONMENT_BRDF_LUT_HEIGHT, ENVIRONMENT_BRDF_LUT_SAMPLE_COUNT, ENVIRONMENT_BRDF_LUT_WIDTH,
};

pub type EnvironmentBrdfLutTexel = [Real; 2];

pub fn build_environment_brdf_lut(size: u32, sample_count: u32) -> Vec<EnvironmentBrdfLutTexel> {
    build_environment_brdf_lut_with_extent(size, size, sample_count)
}

/// split-sum PBR 查找表的 CPU 参考构建入口，用于校验内置设备级 LUT 的视角与粗糙度响应。
/// 生产渲染器上传随程序分发的 LUT 字节；此结果不随单张天空图或 `.zribl` 工件重建。
pub fn build_environment_brdf_lut_with_extent(
    width: u32,
    height: u32,
    sample_count: u32,
) -> Vec<EnvironmentBrdfLutTexel> {
    let width = width.max(1);
    let height = height.max(1);
    let sample_count = sample_count.max(1);
    let mut texels = Vec::with_capacity(width as usize * height as usize);
    for y in 0..height {
        let roughness = (y as Real + 0.5) / height as Real;
        for x in 0..width {
            let no_v = (x as Real + 0.5) / width as Real;
            texels.push(environment_brdf_lut_integrate(
                no_v,
                roughness,
                sample_count,
            ));
        }
    }
    texels
}

pub fn environment_brdf_lut_texel_index(size: u32, x: u32, y: u32) -> usize {
    let size = size.max(1);
    y.min(size - 1) as usize * size as usize + x.min(size - 1) as usize
}

pub fn environment_brdf_lut_integrate(
    no_v: Real,
    roughness: Real,
    sample_count: u32,
) -> EnvironmentBrdfLutTexel {
    let no_v = no_v.clamp(0.001, 1.0);
    let roughness = roughness.clamp(0.0, 1.0);
    let sample_count = sample_count.max(1);
    let sin_theta_v = (1.0 - no_v * no_v).max(0.0).sqrt();
    let view = [sin_theta_v, 0.0, no_v];
    let alpha_squared = roughness * roughness * roughness * roughness;
    let mut scale = 0.0;
    let mut bias = 0.0;

    for sample_index in 0..sample_count {
        let xi = hammersley(sample_index, sample_count);
        let half_vector = importance_sample_ggx(xi, alpha_squared);
        let view_dot_half = dot(view, half_vector).max(0.0);
        let light = normalize_or_zero([
            2.0 * view_dot_half * half_vector[0] - view[0],
            2.0 * view_dot_half * half_vector[1] - view[1],
            2.0 * view_dot_half * half_vector[2] - view[2],
        ]);

        let no_l = light[2].max(0.0);
        let no_h = half_vector[2].max(0.0);
        if no_l > 0.0 && no_h > 0.0 && view_dot_half > 0.0 {
            let visibility = visibility_smith_joint_approx(no_v, no_l, roughness);
            let geometry_visibility = no_l * visibility * (4.0 * view_dot_half / no_h);
            let fresnel = (1.0 - view_dot_half).clamp(0.0, 1.0).powi(5);
            scale += (1.0 - fresnel) * geometry_visibility;
            bias += fresnel * geometry_visibility;
        }
    }

    conserve_perfect_mirror_energy(scale / sample_count as Real, bias / sample_count as Real)
}

fn conserve_perfect_mirror_energy(scale: Real, bias: Real) -> EnvironmentBrdfLutTexel {
    let response = scale + bias;
    if response > 1.0 {
        let normalization = 1.0 / response;
        [scale * normalization, bias * normalization]
    } else {
        [scale, bias]
    }
}

fn hammersley(index: u32, sample_count: u32) -> [Real; 2] {
    [
        index as Real / sample_count.max(1) as Real,
        index.reverse_bits() as Real * 2.328_306_4e-10,
    ]
}

fn importance_sample_ggx(xi: [Real; 2], alpha_squared: Real) -> [Real; 3] {
    let phi = std::f32::consts::TAU * xi[0];
    let cos_theta = ((1.0 - xi[1]) / (1.0 + (alpha_squared - 1.0) * xi[1]).max(0.001)).sqrt();
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    [sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta]
}

fn visibility_smith_joint_approx(no_v: Real, no_l: Real, roughness: Real) -> Real {
    let alpha = roughness * roughness;
    let visibility_v = no_l * (no_v * (1.0 - alpha) + alpha);
    let visibility_l = no_v * (no_l * (1.0 - alpha) + alpha);
    0.5 / (visibility_v + visibility_l)
}

fn normalize_or_zero(value: [Real; 3]) -> [Real; 3] {
    let length_squared = dot(value, value);
    if length_squared <= 0.0 {
        return [0.0, 0.0, 0.0];
    }
    let inverse_length = 1.0 / length_squared.sqrt();
    [
        value[0] * inverse_length,
        value[1] * inverse_length,
        value[2] * inverse_length,
    ]
}

fn dot(lhs: [Real; 3], rhs: [Real; 3]) -> Real {
    lhs[0] * rhs[0] + lhs[1] * rhs[1] + lhs[2] * rhs[2]
}

#[cfg(test)]
#[path = "tests/environment_brdf_lut.rs"]
mod tests;
