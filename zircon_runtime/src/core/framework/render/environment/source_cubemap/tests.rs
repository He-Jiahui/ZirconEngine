use super::*;

#[test]
fn source_cubemap_face_size_clamps_equirect_height_to_power_of_two() {
    assert_eq!(source_cubemap_face_size_from_equirect_height(512), 256);
    assert_eq!(source_cubemap_face_size_from_equirect_height(32), 64);
    assert_eq!(source_cubemap_face_size_from_equirect_height(4096), 1024);
}

#[test]
fn source_cubemap_mip_layout_is_face_major() {
    assert_eq!(source_cubemap_mip_count(4), 3);
    assert_eq!(source_cubemap_sample_count(4, 3), 6 * (16 + 4 + 1));
    assert_eq!(
        source_cubemap_face_mip_offset(4, 3, CubemapFace::PositiveX, 1),
        16
    );
    assert_eq!(
        source_cubemap_face_mip_offset(4, 3, CubemapFace::NegativeX, 0),
        21
    );
}

#[test]
fn source_cubemap_roughness_mip_mapping_matches_shader_contract() {
    let mip_count = 9;
    assert_close(source_cubemap_pmrem_mip_from_roughness(0.0, mip_count), 0.0);
    assert_close(source_cubemap_pmrem_mip_from_roughness(1.0, mip_count), 8.0);
    assert_close(source_cubemap_roughness_from_pmrem_mip(0, mip_count), 0.0);
    assert_close(source_cubemap_roughness_from_pmrem_mip(8, mip_count), 1.0);

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
fn source_cubemap_public_roughness_mip_constants_match_max_face_size() {
    assert_eq!(
        SOURCE_CUBEMAP_ROUGHEST_MIP,
        source_cubemap_mip_count(SOURCE_CUBEMAP_MAX_FACE_SIZE) - 1
    );
    assert_close(
        SOURCE_CUBEMAP_ROUGHNESS_MIP_SCALE,
        SOURCE_CUBEMAP_ROUGHEST_MIP as Real,
    );
}

#[test]
fn source_cubemap_irradiance_mip_prefers_thirty_two_face_source() {
    assert_eq!(source_cubemap_irradiance_mip_level(16, 5), 0);
    assert_eq!(source_cubemap_irradiance_mip_level(64, 7), 1);
    assert_eq!(source_cubemap_irradiance_mip_level(256, 9), 3);
}

#[test]
fn source_cubemap_constant_equirect_preserves_all_mips() {
    let cubemap = build_source_cubemap_from_equirect(4, |_, _| [0.25, 0.5, 0.75, 1.0]);

    assert_eq!(cubemap.face_size(), 4);
    assert_eq!(cubemap.mip_count(), 3);
    for texel in cubemap.texels() {
        assert_vec4_close(*texel, [0.25, 0.5, 0.75, 1.0]);
    }
}

#[test]
fn captured_face_base_level_reuses_source_mips_pmrem_and_sh9() {
    let face_size = 4;
    let mut captured = Vec::new();
    for face in CubemapFace::ALL {
        let value = (face.index() + 1) as Real;
        captured.extend(vec![[value, value * 0.5, value * 0.25, 1.0]; 16]);
    }

    let cubemap = build_source_cubemap_from_captured_faces(face_size, captured);

    assert_eq!(cubemap.face_size(), face_size);
    assert_eq!(cubemap.mip_count(), 3);
    for face in CubemapFace::ALL {
        let expected = (face.index() + 1) as Real;
        assert_vec4_close(
            cubemap.texel(face, 0, 2, 1),
            [expected, expected * 0.5, expected * 0.25, 1.0],
        );
    }
    assert!(cubemap
        .texels()
        .iter()
        .all(|texel| texel.iter().all(|value| value.is_finite())));
    assert!(cubemap
        .irradiance_sh9()
        .iter()
        .flatten()
        .all(|value| value.is_finite()));
}

#[test]
fn captured_face_hash_is_stable_and_tracks_hdr_texels() {
    let first = vec![[1.5, 0.25, 4.0, 1.0]; 6];
    let mut changed = first.clone();
    changed[5][2] = 4.25;

    assert_eq!(
        source_cubemap_capture_hash(1, &first),
        source_cubemap_capture_hash(1, &first)
    );
    assert_ne!(
        source_cubemap_capture_hash(1, &first),
        source_cubemap_capture_hash(1, &changed)
    );
    assert_ne!(
        source_cubemap_capture_hash(1, &first),
        source_cubemap_capture_hash(2, &first)
    );
}

#[test]
fn source_cubemap_sh9_preserves_constant_diffuse_environment() {
    let cubemap = build_source_cubemap_from_equirect(64, |_, _| [0.25, 0.5, 0.75, 1.0]);
    let irradiance = source_cubemap_evaluate_irradiance_sh9(
        cubemap.irradiance_sh9(),
        normalize_or_positive_z([0.25, 1.0, 0.5]),
    );

    assert_vec3_close(irradiance, [0.25, 0.5, 0.75], 0.002);
}

#[test]
fn source_cubemap_sh9_tracks_vertical_environment_gradient() {
    let cubemap = build_source_cubemap_from_equirect(64, |_, v| {
        let sky_weight = 1.0 - v;
        [sky_weight, sky_weight, sky_weight, 1.0]
    });
    let up = source_cubemap_evaluate_irradiance_sh9(cubemap.irradiance_sh9(), [0.0, 1.0, 0.0]);
    let down = source_cubemap_evaluate_irradiance_sh9(cubemap.irradiance_sh9(), [0.0, -1.0, 0.0]);

    assert!(
        up[0] > down[0] + 0.2,
        "up-facing diffuse irradiance should see the brighter sky, up={up:?} down={down:?}"
    );
}

#[test]
fn source_cubemap_cmft_pmrem_mips_blur_high_frequency_source() {
    let cubemap = build_source_cubemap_from_equirect(8, |u, _| {
        if u < 0.5 {
            [0.0, 0.0, 0.0, 1.0]
        } else {
            [1.0, 1.0, 1.0, 1.0]
        }
    });
    let last_mip = cubemap.mip_count() - 1;
    let last = cubemap.texel(CubemapFace::PositiveX, last_mip, 0, 0);

    assert_eq!(cubemap.texel(CubemapFace::NegativeX, 0, 4, 4)[0], 0.0);
    assert_eq!(cubemap.texel(CubemapFace::PositiveX, 0, 4, 4)[0], 1.0);
    assert!(
        last[0] > 0.1 && last[0] < 0.9,
        "lowest radiance mip should be blurred toward the environment average, got {last:?}"
    );
    for face in CubemapFace::ALL {
        assert_eq!(cubemap.texel(face, last_mip, 0, 0), last);
    }
}

#[test]
fn source_cubemap_saturated_roughness_mip_uses_cosine_convolution() {
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
        source_cubemap_pmrem_mip_from_roughness(1.0, cubemap.mip_count()).round() as u32;
    assert!(
        saturated_mip > 0,
        "roughness=1 should select a PMREM mip below the base level"
    );
    let mip_size = source_cubemap_mip_size(cubemap.face_size(), saturated_mip);
    let previous_variance = mip_luma_variance(&cubemap, saturated_mip - 1);
    let saturated_variance = mip_luma_variance(&cubemap, saturated_mip);
    let mut max_downsample_luma_delta: Real = 0.0;

    for face in CubemapFace::ALL {
        for y in 0..mip_size {
            for x in 0..mip_size {
                let direction = cubemap_texel_direction(face, x, y, mip_size);
                let ordinary_downsample = sample_cubemap_linear_at_mip(
                    cubemap.texels(),
                    cubemap.face_size(),
                    cubemap.mip_count(),
                    direction,
                    saturated_mip - 1,
                );
                let actual = cubemap.texel(face, saturated_mip, x, y);
                max_downsample_luma_delta = max_downsample_luma_delta
                    .max((luma4(actual) - luma4(ordinary_downsample)).abs());
            }
        }
    }

    assert!(
        max_downsample_luma_delta > 0.025,
        "roughness=1 PMREM mip should be a source-space cosine convolution, not ordinary previous-mip downsample, delta={max_downsample_luma_delta}"
    );
    assert!(
        saturated_variance < previous_variance * 0.75,
        "roughness=1 PMREM mip should further blur high-frequency energy, previous={previous_variance} saturated={saturated_variance}"
    );
}

#[test]
fn source_cubemap_cmft_pmrem_reduces_mip_luma_variance() {
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
    let rough_variance = mip_luma_variance(&cubemap, cubemap.mip_count().saturating_sub(2));

    assert!(
        rough_variance < base_variance * 0.45,
        "rough PMREM mip should reduce high-frequency luma variance, base={base_variance} rough={rough_variance}"
    );
}

#[test]
fn source_cubemap_samples_equirect_uv_from_cube_face_direction() {
    let cubemap = build_source_cubemap_from_equirect(3, |u, v| [u, v, 0.0, 1.0]);

    assert_vec4_close(
        cubemap.texel(CubemapFace::PositiveZ, 0, 1, 1),
        [0.5, 0.5, 0.0, 1.0],
    );
    assert_vec4_close(
        cubemap.texel(CubemapFace::PositiveX, 0, 1, 1),
        [0.75, 0.5, 0.0, 1.0],
    );
}

#[test]
fn source_cubemap_linear_sampling_bleeds_across_face_edges() {
    let face_size = 4;
    let mip_count = 1;
    let mut texels = vec![[0.0, 0.0, 0.0, 1.0]; source_cubemap_sample_count(face_size, mip_count)];
    fill_face_texels(
        &mut texels,
        face_size,
        mip_count,
        CubemapFace::PositiveX,
        [1.0, 0.0, 0.0, 1.0],
    );
    fill_face_texels(
        &mut texels,
        face_size,
        mip_count,
        CubemapFace::PositiveZ,
        [0.0, 1.0, 0.0, 1.0],
    );
    let direction = cubemap_direction_from_scaled_uv(CubemapFace::PositiveX, [-0.98, 0.0]);

    let color = sample_source_cubemap_trilinear(&texels, face_size, mip_count, direction, 0.0);

    assert!(
        color[0] < 0.9 && color[1] > 0.05,
        "sampling near +X left edge should include +Z neighbor texels instead of clamping inside +X, color={color:?}"
    );
}

fn fill_face_texels(
    texels: &mut [[Real; 4]],
    face_size: u32,
    mip_count: u32,
    face: CubemapFace,
    color: [Real; 4],
) {
    let offset = source_cubemap_face_mip_offset(face_size, mip_count, face, 0);
    for y in 0..face_size {
        for x in 0..face_size {
            texels[offset + y as usize * face_size as usize + x as usize] = color;
        }
    }
}

fn assert_vec4_close(actual: [Real; 4], expected: [Real; 4]) {
    for index in 0..4 {
        assert!(
            (actual[index] - expected[index]).abs() <= 0.00001,
            "component {index}: actual={actual:?} expected={expected:?}"
        );
    }
}

fn assert_close(actual: Real, expected: Real) {
    assert!(
        (actual - expected).abs() <= 0.00001,
        "actual={actual} expected={expected}"
    );
}

fn assert_vec3_close(actual: [Real; 3], expected: [Real; 3], tolerance: Real) {
    for index in 0..3 {
        assert!(
            (actual[index] - expected[index]).abs() <= tolerance,
            "component {index}: actual={actual:?} expected={expected:?}"
        );
    }
}

fn mip_luma_variance(cubemap: &SourceCubemapMipChain, mip_level: u32) -> Real {
    let mip_size = source_cubemap_mip_size(cubemap.face_size(), mip_level);
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    let mut count = 0.0;
    for face in CubemapFace::ALL {
        for y in 0..mip_size {
            for x in 0..mip_size {
                let texel = cubemap.texel(face, mip_level, x, y);
                let luma = luma4(texel);
                sum += luma;
                sum_sq += luma * luma;
                count += 1.0;
            }
        }
    }
    let mean = sum / count;
    sum_sq / count - mean * mean
}

fn luma4(texel: [Real; 4]) -> Real {
    0.2126 * texel[0] + 0.7152 * texel[1] + 0.0722 * texel[2]
}
