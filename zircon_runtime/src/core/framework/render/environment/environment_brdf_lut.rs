use crate::core::math::Real;

pub const ENVIRONMENT_BRDF_LUT_WIDTH: u32 = 128;
pub const ENVIRONMENT_BRDF_LUT_HEIGHT: u32 = 32;
pub const ENVIRONMENT_BRDF_LUT_SAMPLE_COUNT: u32 = 128;

pub type EnvironmentBrdfLutTexel = [Real; 2];

pub fn build_environment_brdf_lut(size: u32, sample_count: u32) -> Vec<EnvironmentBrdfLutTexel> {
    build_environment_brdf_lut_with_extent(size, size, sample_count)
}

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
            let geometry = geometry_smith_ibl(no_v, no_l, roughness);
            let geometry_visibility = geometry * view_dot_half / (no_h * no_v).max(0.001);
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

fn geometry_smith_ibl(no_v: Real, no_l: Real, roughness: Real) -> Real {
    geometry_schlick_ggx_ibl(no_v, roughness) * geometry_schlick_ggx_ibl(no_l, roughness)
}

fn geometry_schlick_ggx_ibl(no: Real, roughness: Real) -> Real {
    let k = roughness * roughness * 0.5;
    no / (no * (1.0 - k) + k).max(0.001)
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
mod tests {
    use super::*;

    #[test]
    fn environment_brdf_lut_corner_values_match_split_sum_contract() {
        let sharp_normal = environment_brdf_lut_integrate(1.0, 0.0, 1024);
        assert!(sharp_normal[0] > 0.99, "{sharp_normal:?}");
        assert!(sharp_normal[1] < 0.01, "{sharp_normal:?}");

        let rough_normal = environment_brdf_lut_integrate(1.0, 1.0, 1024);
        assert!(rough_normal[0] + rough_normal[1] < 0.5, "{rough_normal:?}");
    }

    #[test]
    fn environment_brdf_lut_builder_outputs_finite_rg_texels() {
        let size = 8;
        let texels = build_environment_brdf_lut(size, 64);
        assert_eq!(texels.len(), size as usize * size as usize);
        for texel in texels {
            assert!(texel[0].is_finite(), "{texel:?}");
            assert!(texel[1].is_finite(), "{texel:?}");
            assert!(texel[0] >= 0.0, "{texel:?}");
            assert!(texel[1] >= 0.0, "{texel:?}");
        }
    }

    #[test]
    fn runtime_lut_matches_the_unreal_preintegrated_gf_work_scale() {
        let texels = build_environment_brdf_lut_with_extent(
            ENVIRONMENT_BRDF_LUT_WIDTH,
            ENVIRONMENT_BRDF_LUT_HEIGHT,
            ENVIRONMENT_BRDF_LUT_SAMPLE_COUNT,
        );

        assert_eq!(texels.len(), 128 * 32);
        assert_eq!(
            texels.len() as u32 * ENVIRONMENT_BRDF_LUT_SAMPLE_COUNT,
            524_288
        );
    }

    #[test]
    fn runtime_sample_count_stays_close_to_a_high_sample_reference() {
        let mut total_error = 0.0;
        let mut maximum_error = 0.0_f32;
        let mut channel_count = 0;
        for y in 0..16 {
            let roughness = (y as Real + 0.5) / 16.0;
            for x in 0..16 {
                let no_v = (x as Real + 0.5) / 16.0;
                let runtime = environment_brdf_lut_integrate(
                    no_v,
                    roughness,
                    ENVIRONMENT_BRDF_LUT_SAMPLE_COUNT,
                );
                let reference = environment_brdf_lut_integrate(no_v, roughness, 4_096);
                for channel in 0..2 {
                    let error = (runtime[channel] - reference[channel]).abs();
                    total_error += error;
                    maximum_error = maximum_error.max(error);
                    channel_count += 1;
                }
            }
        }

        let mean_error = total_error / channel_count as Real;
        assert!(mean_error <= 0.003, "mean absolute error={mean_error}");
        assert!(
            maximum_error <= 0.02,
            "maximum absolute error={maximum_error}"
        );
    }

    #[test]
    fn environment_brdf_lut_conserves_smooth_perfect_mirror_grazing_energy() {
        for no_v in [0.001, 0.005, 0.01, 0.05, 0.1] {
            let texel = environment_brdf_lut_integrate(no_v, 0.0, 4096);
            assert!(
                texel[0] + texel[1] <= 1.0001,
                "no_v={no_v}, texel={texel:?}"
            );
        }
    }
}
