use zircon_runtime::core::framework::render::{
    build_source_cubemap_from_equirect, cubemap_direction_from_scaled_uv,
    cubemap_face_scaled_uv_from_direction, cubemap_scaled_uv_for_texel,
    source_cubemap_evaluate_irradiance_sh9, source_cubemap_face_mip_offset,
    source_cubemap_irradiance_mip_level, source_cubemap_mip_size,
    source_cubemap_pmrem_mip_from_roughness, source_cubemap_roughness_from_pmrem_mip, CubemapFace,
    SourceCubemapMipChain,
};

#[test]
fn runtime_environment_source_cubemap_pmrem_roughness_mapping_matches_shader_contract() {
    let mip_count = 8;

    assert_close(source_cubemap_pmrem_mip_from_roughness(1.0, mip_count), 5.0);
    assert!(source_cubemap_roughness_from_pmrem_mip(4, mip_count) < 1.0);
    assert_close(source_cubemap_roughness_from_pmrem_mip(5, mip_count), 1.0);
    assert_close(source_cubemap_roughness_from_pmrem_mip(6, mip_count), 1.0);
    assert_close(source_cubemap_roughness_from_pmrem_mip(7, mip_count), 1.0);

    let mut previous = 0.0;
    for mip in 1..mip_count {
        let roughness = source_cubemap_roughness_from_pmrem_mip(mip, mip_count);
        assert!(
            roughness >= previous,
            "roughness should increase with mip level, mip={mip} roughness={roughness} previous={previous}"
        );
        previous = roughness;
    }
}

#[test]
fn runtime_environment_source_cubemap_pmrem_preserves_constant_environment() {
    let cubemap = build_source_cubemap_from_equirect(4, |_, _| [0.25, 0.5, 0.75, 1.0]);

    for texel in cubemap.source_texels() {
        assert_texel_close(*texel, [0.25, 0.5, 0.75, 1.0]);
    }
    for texel in cubemap.pmrem_texels() {
        assert_texel_close(*texel, [0.25, 0.5, 0.75, 1.0]);
    }
}

#[test]
fn runtime_environment_source_cubemap_preserves_hdr_values_for_float_upload_path() {
    let cubemap = build_source_cubemap_from_equirect(4, |_, _| [2.5, 1.25, 0.75, 1.0]);

    for texel in cubemap.source_texels() {
        assert_texel_close(*texel, [2.5, 1.25, 0.75, 1.0]);
    }
    for texel in cubemap.pmrem_texels() {
        assert_texel_close(*texel, [2.5, 1.25, 0.75, 1.0]);
    }
}

#[test]
fn runtime_environment_source_cubemap_sh9_preserves_constant_diffuse_environment() {
    let cubemap = build_source_cubemap_from_equirect(64, |_, _| [0.25, 0.5, 0.75, 1.0]);
    let irradiance =
        source_cubemap_evaluate_irradiance_sh9(cubemap.irradiance_sh9(), [0.25, 1.0, 0.5]);

    assert_rgb_close(irradiance, [0.25, 0.5, 0.75], 0.002);
}

#[test]
fn runtime_environment_source_cubemap_sh9_tracks_vertical_gradient() {
    assert_eq!(source_cubemap_irradiance_mip_level(256, 9), 3);

    let cubemap = build_source_cubemap_from_equirect(64, |_, v| {
        let sky_weight = 1.0 - v;
        [sky_weight, sky_weight, sky_weight, 1.0]
    });
    let up = source_cubemap_evaluate_irradiance_sh9(cubemap.irradiance_sh9(), [0.0, 1.0, 0.0]);
    let down = source_cubemap_evaluate_irradiance_sh9(cubemap.irradiance_sh9(), [0.0, -1.0, 0.0]);

    assert!(
        up[0] > down[0] + 0.2,
        "up-facing diffuse irradiance should be brighter than down-facing irradiance, up={up:?} down={down:?}"
    );
}

#[test]
fn runtime_environment_source_cubemap_pmrem_blurs_high_frequency_environment() {
    let cubemap = build_source_cubemap_from_equirect(16, |u, v| {
        let cell_x = (u * 24.0).floor() as i32;
        let cell_y = (v * 12.0).floor() as i32;
        if (cell_x + cell_y) & 1 == 0 {
            [0.0, 0.0, 0.0, 1.0]
        } else {
            [1.0, 1.0, 1.0, 1.0]
        }
    });
    let base_variance = mip_luma_variance(&cubemap, 0);
    let rough_mip =
        source_cubemap_pmrem_mip_from_roughness(1.0, cubemap.pmrem_mip_count()).round() as u32;
    let rough_variance = mip_luma_variance(&cubemap, rough_mip);
    let pmrem_comparison_mip =
        source_cubemap_pmrem_mip_from_roughness(0.5, cubemap.pmrem_mip_count()).round() as u32;
    let pmrem_comparison_size =
        source_cubemap_mip_size(cubemap.pmrem_face_size(), pmrem_comparison_mip);
    let source_comparison_mip = ((cubemap.source_face_size() / pmrem_comparison_size.max(1))
        .max(1)
        .ilog2())
    .min(cubemap.source_mip_count().saturating_sub(1));
    let source_mip_variance = source_mip_luma_variance(&cubemap, source_comparison_mip);
    let pmrem_comparison_variance = mip_luma_variance(&cubemap, pmrem_comparison_mip);

    assert!(
        rough_variance < base_variance * 0.45,
        "rough PMREM mip should reduce high-frequency luma variance, base={base_variance} rough={rough_variance}"
    );
    assert!(
        pmrem_comparison_variance < source_mip_variance * 0.75,
        "GGX PMREM mip should be blurrier than the regular source mip at the same face resolution, source_mip={source_comparison_mip} pmrem_mip={pmrem_comparison_mip} face_size={pmrem_comparison_size} source={source_mip_variance} pmrem={pmrem_comparison_variance}"
    );
}

#[test]
fn runtime_environment_source_cubemap_source_mips_blur_high_frequency_environment() {
    let cubemap = build_source_cubemap_from_equirect(64, |u, v| {
        let cell_x = (u * 61.0).floor() as i32;
        let cell_y = (v * 31.0).floor() as i32;
        let luma = if (cell_x + cell_y) & 1 == 0 { 0.0 } else { 1.0 };
        [luma, luma, luma, 1.0]
    });

    let base_variance = source_mip_luma_variance(&cubemap, 0);
    let mid_mip = cubemap
        .source_mip_count()
        .min(cubemap.pmrem_mip_count())
        .saturating_sub(3);
    let source_mid_variance = source_mip_luma_variance(&cubemap, mid_mip);
    let pmrem_mid_variance = mip_luma_variance(&cubemap, mid_mip);

    assert!(
        source_mid_variance < base_variance * 0.5,
        "source angular mip should reduce high-frequency luma variance, base={base_variance} source_mid={source_mid_variance}"
    );
    assert!(
        source_mid_variance > pmrem_mid_variance * 1.2,
        "source angular mip should stay sharper than the specular PMREM at the same level, source_mid={source_mid_variance} pmrem_mid={pmrem_mid_variance}"
    );
}

#[test]
fn runtime_environment_source_cubemap_source_roughest_mip_averages_all_faces() {
    let cubemap = build_source_cubemap_from_equirect(32, |u, v| {
        let luma = if u < 0.5 { 0.15 } else { 1.6 } + v * 0.45;
        [luma, luma * 0.8, luma * 0.55, 1.0]
    });
    let last_mip = cubemap.source_mip_count().saturating_sub(1);
    let first = source_texel(&cubemap, CubemapFace::PositiveX, last_mip, 0, 0);

    for face in CubemapFace::ALL {
        assert_texel_close(source_texel(&cubemap, face, last_mip, 0, 0), first);
    }
}

#[test]
fn runtime_environment_source_cubemap_pmrem_mips_progressively_blur_high_frequency_environment() {
    let cubemap = build_source_cubemap_from_equirect(64, |u, v| {
        let cell_x = (u * 53.0).floor() as i32;
        let cell_y = (v * 29.0).floor() as i32;
        let checker = if (cell_x + cell_y) & 1 == 0 {
            0.08
        } else {
            1.0
        };
        let sun_distance_u = wrapped_unit_distance(u, 0.62);
        let sun_distance_v = v - 0.36;
        let sun = if sun_distance_u * sun_distance_u + sun_distance_v * sun_distance_v < 0.0012 {
            5.0
        } else {
            0.0
        };
        let luma = checker + sun;
        [luma, luma, luma, 1.0]
    });
    let variances: Vec<_> = (0..cubemap.pmrem_mip_count())
        .map(|mip| mip_luma_variance(&cubemap, mip))
        .collect();

    assert!(
        variances[1] < variances[0] * 0.9,
        "first PMREM mip should already reduce high-frequency variance, variances={variances:?}"
    );

    let rough_mip =
        source_cubemap_pmrem_mip_from_roughness(1.0, cubemap.pmrem_mip_count()).round() as u32;
    for mip in 2..=rough_mip {
        let previous = variances[mip as usize - 1];
        let current = variances[mip as usize];
        assert!(
            current <= previous * 0.97 + 0.0001,
            "PMREM variance should keep falling as roughness selects coarser mips, mip={mip} previous={previous} current={current} variances={variances:?}"
        );
    }

    let rough_mip = rough_mip as usize;
    assert!(
        variances[rough_mip] < variances[0] * 0.12,
        "rough PMREM mip should be heavily blurred relative to mip0, base={} rough={} variances={variances:?}",
        variances[0], variances[rough_mip]
    );
}

#[test]
fn runtime_environment_source_cubemap_saturated_roughness_mip_uses_cosine_convolution() {
    let cubemap = build_source_cubemap_from_equirect(64, |u, v| {
        let stripe = if (u * 37.0).floor() as i32 & 1 == 0 {
            0.1
        } else {
            1.2
        };
        let horizon = (1.0 - (v - 0.52).abs() * 8.0).max(0.0) * 2.0;
        let luma = stripe + horizon;
        [luma, luma * 0.85, luma * 0.65, 1.0]
    });
    let saturated_mip =
        source_cubemap_pmrem_mip_from_roughness(1.0, cubemap.pmrem_mip_count()).round() as u32;
    assert!(
        saturated_mip > 0,
        "roughness=1 should select a PMREM mip below the base level"
    );
    let mip_size = source_cubemap_mip_size(cubemap.pmrem_face_size(), saturated_mip);
    let previous_variance = mip_luma_variance(&cubemap, saturated_mip - 1);
    let saturated_variance = mip_luma_variance(&cubemap, saturated_mip);
    let mut max_downsample_luma_delta = 0.0_f32;

    for face in CubemapFace::ALL {
        for y in 0..mip_size {
            for x in 0..mip_size {
                let direction = cubemap_direction_from_scaled_uv(
                    face,
                    cubemap_scaled_uv_for_texel(x, y, mip_size),
                );
                let ordinary_downsample =
                    sample_pmrem_linear_at_mip(&cubemap, direction, saturated_mip - 1);
                let actual = cubemap.pmrem_texel(face, saturated_mip, x, y);
                max_downsample_luma_delta =
                    max_downsample_luma_delta.max((luma(actual) - luma(ordinary_downsample)).abs());
            }
        }
    }

    assert!(
        max_downsample_luma_delta > 0.025,
        "roughness=1 PMREM mip should be source-space cosine convolution, not ordinary previous-mip downsample, delta={max_downsample_luma_delta}"
    );
    assert!(
        saturated_variance < previous_variance * 0.75,
        "roughness=1 PMREM mip should further blur high-frequency energy, previous={previous_variance} saturated={saturated_variance}"
    );
}

#[test]
fn runtime_environment_source_cubemap_pmrem_roughest_mip_averages_all_faces() {
    let cubemap = build_source_cubemap_from_equirect(32, |u, v| {
        let luma = if u < 0.5 { 0.15 } else { 1.4 } + v * 0.35;
        [luma, luma * 0.75, luma * 0.5, 1.0]
    });
    let last_mip = cubemap.pmrem_mip_count().saturating_sub(1);
    let average = cubemap.pmrem_texel(CubemapFace::PositiveX, last_mip, 0, 0);

    for face in CubemapFace::ALL {
        assert_texel_close(cubemap.pmrem_texel(face, last_mip, 0, 0), average);
    }
}

#[test]
fn runtime_environment_source_cubemap_pmrem_rough_mips_reduce_cube_seam_energy() {
    let cubemap = build_source_cubemap_from_equirect(64, |u, v| {
        let wave_a = (std::f32::consts::TAU * u * 17.0).sin();
        let wave_b = (std::f32::consts::TAU * (u * 11.0 + v * 7.0)).cos();
        let wave_c = (std::f32::consts::PI * v * 9.0).sin();
        let luma = 0.55 + wave_a * 0.22 + wave_b * 0.16 + wave_c * 0.12;
        [luma, luma * 0.85, luma * 0.7, 1.0]
    });
    let base = pmrem_seam_luma_stats(&cubemap, 0);
    let mid_mip =
        source_cubemap_pmrem_mip_from_roughness(0.5, cubemap.pmrem_mip_count()).round() as u32;
    let rough_mip =
        source_cubemap_pmrem_mip_from_roughness(1.0, cubemap.pmrem_mip_count()).round() as u32;
    let mid = pmrem_seam_luma_stats(&cubemap, mid_mip);
    let rough = pmrem_seam_luma_stats(&cubemap, rough_mip);

    assert!(
        mid.mean < base.mean * 0.9,
        "mid PMREM mip should reduce cube-edge seam energy, base={base:?} mid={mid:?} rough={rough:?}"
    );
    assert!(
        rough.mean <= mid.mean * 0.9 + 0.001,
        "rougher PMREM mip should continue reducing mean seam energy, base={base:?} mid={mid:?} rough={rough:?}"
    );
    assert!(
        rough.max < base.max * 0.75,
        "rough PMREM mip should reduce worst cube-edge seam energy, base={base:?} mid={mid:?} rough={rough:?}"
    );
}

fn source_mip_luma_variance(cubemap: &SourceCubemapMipChain, mip_level: u32) -> f32 {
    mip_luma_variance_from_texels(
        cubemap.source_texels(),
        cubemap.source_face_size(),
        cubemap.source_mip_count(),
        mip_level,
    )
}

fn source_texel(
    cubemap: &SourceCubemapMipChain,
    face: CubemapFace,
    mip_level: u32,
    x: u32,
    y: u32,
) -> [f32; 4] {
    let mip_size = source_cubemap_mip_size(cubemap.source_face_size(), mip_level);
    let offset = source_cubemap_face_mip_offset(
        cubemap.source_face_size(),
        cubemap.source_mip_count(),
        face,
        mip_level,
    );
    cubemap.source_texels()[offset + y as usize * mip_size as usize + x as usize]
}

fn mip_luma_variance(cubemap: &SourceCubemapMipChain, mip_level: u32) -> f32 {
    mip_luma_variance_from_texels(
        cubemap.pmrem_texels(),
        cubemap.pmrem_face_size(),
        cubemap.pmrem_mip_count(),
        mip_level,
    )
}

fn mip_luma_variance_from_texels(
    texels: &[[f32; 4]],
    face_size: u32,
    mip_count: u32,
    mip_level: u32,
) -> f32 {
    let mip_size = source_cubemap_mip_size(face_size, mip_level);
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    let mut count = 0.0;
    for face in CubemapFace::ALL {
        for y in 0..mip_size {
            for x in 0..mip_size {
                let offset =
                    zircon_runtime::core::framework::render::source_cubemap_face_mip_offset(
                        face_size, mip_count, face, mip_level,
                    );
                let texel = texels[offset + y as usize * mip_size as usize + x as usize];
                let luma = 0.2126 * texel[0] + 0.7152 * texel[1] + 0.0722 * texel[2];
                sum += luma;
                sum_sq += luma * luma;
                count += 1.0;
            }
        }
    }
    let mean = sum / count;
    sum_sq / count - mean * mean
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.00001,
        "actual={actual} expected={expected}"
    );
}

fn wrapped_unit_distance(a: f32, b: f32) -> f32 {
    let distance = (a - b).abs();
    distance.min(1.0 - distance)
}

#[derive(Clone, Copy, Debug)]
struct SeamLumaStats {
    mean: f32,
    max: f32,
}

fn pmrem_seam_luma_stats(cubemap: &SourceCubemapMipChain, mip_level: u32) -> SeamLumaStats {
    let mip_size = source_cubemap_mip_size(cubemap.pmrem_face_size(), mip_level);
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
                let current = cubemap.pmrem_texel(face, mip_level, x, y);
                let (neighbor_face, neighbor_x, neighbor_y) =
                    side.neighbor_texel(face, index, mip_size);
                let neighbor =
                    cubemap.pmrem_texel(neighbor_face, mip_level, neighbor_x, neighbor_y);
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

fn sample_pmrem_linear_at_mip(
    cubemap: &SourceCubemapMipChain,
    direction: [f32; 3],
    mip_level: u32,
) -> [f32; 4] {
    let mip_size = source_cubemap_mip_size(cubemap.pmrem_face_size(), mip_level);
    let (face, scaled_uv) = cubemap_face_scaled_uv_from_direction(direction);
    let u = (scaled_uv[0] * 0.5 + 0.5) * mip_size as f32 - 0.5;
    let v = (scaled_uv[1] * 0.5 + 0.5) * mip_size as f32 - 0.5;
    let x0 = u.floor().clamp(0.0, mip_size.saturating_sub(1) as f32) as u32;
    let y0 = v.floor().clamp(0.0, mip_size.saturating_sub(1) as f32) as u32;
    let x1 = (x0 + 1).min(mip_size.saturating_sub(1));
    let y1 = (y0 + 1).min(mip_size.saturating_sub(1));
    let tx = (u - x0 as f32).clamp(0.0, 1.0);
    let ty = (v - y0 as f32).clamp(0.0, 1.0);
    let c00 = cubemap.pmrem_texel(face, mip_level, x0, y0);
    let c10 = cubemap.pmrem_texel(face, mip_level, x1, y0);
    let c01 = cubemap.pmrem_texel(face, mip_level, x0, y1);
    let c11 = cubemap.pmrem_texel(face, mip_level, x1, y1);
    lerp_vec4(lerp_vec4(c00, c10, tx), lerp_vec4(c01, c11, tx), ty)
}

fn lerp_vec4(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

fn assert_texel_close(actual: [f32; 4], expected: [f32; 4]) {
    for index in 0..4 {
        assert!(
            (actual[index] - expected[index]).abs() <= 0.00001,
            "component {index}: actual={actual:?} expected={expected:?}"
        );
    }
}

fn assert_rgb_close(actual: [f32; 3], expected: [f32; 3], tolerance: f32) {
    for index in 0..3 {
        assert!(
            (actual[index] - expected[index]).abs() <= tolerance,
            "component {index}: actual={actual:?} expected={expected:?}"
        );
    }
}
