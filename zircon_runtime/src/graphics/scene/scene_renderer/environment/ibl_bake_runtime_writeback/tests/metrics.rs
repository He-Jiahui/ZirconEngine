use crate::core::framework::render::{
    cubemap_direction_from_scaled_uv, cubemap_face_scaled_uv_from_direction,
    cubemap_scaled_uv_for_texel, source_cubemap_mip_size, CubemapFace, SourceCubemapIrradianceCube,
    SourceCubemapMipChain,
};

pub(super) fn synthetic_seam_stress_environment(u: f32, v: f32) -> [f32; 4] {
    let wave_a = (std::f32::consts::TAU * u * 17.0).sin();
    let wave_b = (std::f32::consts::TAU * (u * 11.0 + v * 7.0)).cos();
    let wave_c = (std::f32::consts::PI * v * 9.0).sin();
    let luma = 0.55 + wave_a * 0.22 + wave_b * 0.16 + wave_c * 0.12;
    [luma, luma * 0.85, luma * 0.7, 1.0]
}

pub(super) fn synthetic_irradiance_environment(u: f32, v: f32) -> [f32; 4] {
    let phi = std::f32::consts::TAU * u - std::f32::consts::PI;
    let y = (1.0 - 2.0 * v).clamp(-1.0, 1.0);
    let radius = (1.0 - y * y).max(0.0).sqrt();
    let x = radius * phi.sin();
    let z = radius * phi.cos();
    let luma = (0.46 + x * 0.18 + y * 0.14 - z * 0.11).clamp(0.08, 0.92);
    [luma, luma * 0.82 + 0.04, luma * 0.62 + 0.08, 1.0]
}

#[derive(Clone, Copy, Debug)]
pub(super) struct IrradianceCubeDirectionalStats {
    pub(super) computed_mean: f32,
    pub(super) reference_mean: f32,
    pub(super) normalized_rms: f32,
    pub(super) correlation: f32,
    pub(super) computed_dynamic_range: f32,
    pub(super) reference_dynamic_range: f32,
}

pub(super) fn irradiance_cube_directional_stats(
    computed: &SourceCubemapIrradianceCube,
    reference: &SourceCubemapIrradianceCube,
) -> IrradianceCubeDirectionalStats {
    assert_eq!(computed.face_size(), reference.face_size());
    assert_eq!(computed.texels().len(), reference.texels().len());

    let mut computed_luma = Vec::with_capacity(computed.texels().len());
    let mut reference_luma = Vec::with_capacity(reference.texels().len());
    let mut computed_sum = 0.0;
    let mut reference_sum = 0.0;
    let mut computed_min = f32::MAX;
    let mut computed_max = f32::MIN;
    let mut reference_min = f32::MAX;
    let mut reference_max = f32::MIN;

    for (computed_texel, reference_texel) in computed.texels().iter().zip(reference.texels().iter())
    {
        let computed_value = luma3(*computed_texel);
        let reference_value = luma3(*reference_texel);
        computed_sum += computed_value;
        reference_sum += reference_value;
        computed_min = computed_min.min(computed_value);
        computed_max = computed_max.max(computed_value);
        reference_min = reference_min.min(reference_value);
        reference_max = reference_max.max(reference_value);
        computed_luma.push(computed_value);
        reference_luma.push(reference_value);
    }

    let count = computed_luma.len() as f32;
    let computed_mean = computed_sum / count;
    let reference_mean = reference_sum / count;
    let mut rms_sum = 0.0;
    let mut covariance = 0.0;
    let mut computed_variance = 0.0;
    let mut reference_variance = 0.0;

    for (computed_value, reference_value) in computed_luma.iter().zip(reference_luma.iter()) {
        let computed_normalized = *computed_value / computed_mean.max(f32::EPSILON);
        let reference_normalized = *reference_value / reference_mean.max(f32::EPSILON);
        let delta = computed_normalized - reference_normalized;
        let computed_centered = computed_normalized - 1.0;
        let reference_centered = reference_normalized - 1.0;
        rms_sum += delta * delta;
        covariance += computed_centered * reference_centered;
        computed_variance += computed_centered * computed_centered;
        reference_variance += reference_centered * reference_centered;
    }

    let correlation =
        covariance / (computed_variance.sqrt() * reference_variance.sqrt()).max(f32::EPSILON);
    IrradianceCubeDirectionalStats {
        computed_mean,
        reference_mean,
        normalized_rms: (rms_sum / count).sqrt(),
        correlation,
        computed_dynamic_range: computed_max - computed_min,
        reference_dynamic_range: reference_max - reference_min,
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SeamLumaStats {
    pub(super) mean: f32,
    pub(super) max: f32,
}

pub(super) fn pmrem_seam_luma_stats(
    cubemap: &SourceCubemapMipChain,
    mip_level: u32,
) -> SeamLumaStats {
    let mip_size = source_cubemap_mip_size(cubemap.face_size(), mip_level);
    let mut sum = 0.0;
    let mut max = 0.0_f32;
    let mut count = 0.0;

    for face in CubemapFace::ALL {
        for side in CubeEdgeSide::ALL {
            let sample_start = if mip_size > 2 { 1 } else { 0 };
            let sample_end = if mip_size > 2 {
                mip_size.saturating_sub(1)
            } else {
                mip_size
            };
            for index in sample_start..sample_end {
                let (x, y) = side.edge_texel(index, mip_size);
                let current = cubemap.texel(face, mip_level, x, y);
                let (neighbor_face, neighbor_x, neighbor_y) =
                    side.neighbor_texel(face, index, mip_size);
                let neighbor = cubemap.texel(neighbor_face, mip_level, neighbor_x, neighbor_y);
                let delta = (luma(current) - luma(neighbor)).abs();
                sum += delta;
                max = max.max(delta);
                count += 1.0;
            }
        }
    }

    SeamLumaStats {
        mean: sum / count,
        max,
    }
}

#[derive(Clone, Copy, Debug)]
enum CubeEdgeSide {
    Left,
    Right,
    Top,
    Bottom,
}

impl CubeEdgeSide {
    const ALL: [Self; 4] = [Self::Left, Self::Right, Self::Top, Self::Bottom];

    fn edge_texel(self, index: u32, size: u32) -> (u32, u32) {
        match self {
            Self::Left => (0, index),
            Self::Right => (size.saturating_sub(1), index),
            Self::Top => (index, 0),
            Self::Bottom => (index, size.saturating_sub(1)),
        }
    }

    fn neighbor_texel(self, face: CubemapFace, index: u32, size: u32) -> (CubemapFace, u32, u32) {
        let edge_uv = match self {
            Self::Left => [
                -1.0 - 1.0 / size as f32,
                cubemap_scaled_uv_for_texel(0, index, size)[1],
            ],
            Self::Right => [
                1.0 + 1.0 / size as f32,
                cubemap_scaled_uv_for_texel(size.saturating_sub(1), index, size)[1],
            ],
            Self::Top => [
                cubemap_scaled_uv_for_texel(index, 0, size)[0],
                -1.0 - 1.0 / size as f32,
            ],
            Self::Bottom => [
                cubemap_scaled_uv_for_texel(index, size.saturating_sub(1), size)[0],
                1.0 + 1.0 / size as f32,
            ],
        };
        let direction = cubemap_direction_from_scaled_uv(face, edge_uv);
        let (neighbor_face, neighbor_uv) = cubemap_face_scaled_uv_from_direction(direction);
        (
            neighbor_face,
            texel_coord_from_scaled_axis(neighbor_uv[0], size),
            texel_coord_from_scaled_axis(neighbor_uv[1], size),
        )
    }
}

fn texel_coord_from_scaled_axis(scaled_axis: f32, size: u32) -> u32 {
    (((scaled_axis * 0.5 + 0.5) * size as f32 - 0.5).round() as i32)
        .clamp(0, size.saturating_sub(1) as i32) as u32
}

fn luma(texel: [f32; 4]) -> f32 {
    0.2126 * texel[0] + 0.7152 * texel[1] + 0.0722 * texel[2]
}

fn luma3(texel: [f32; 3]) -> f32 {
    0.2126 * texel[0] + 0.7152 * texel[1] + 0.0722 * texel[2]
}
