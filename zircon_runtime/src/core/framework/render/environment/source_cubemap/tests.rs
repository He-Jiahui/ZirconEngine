use super::*;

mod projection;

#[test]
fn cloned_mip_chain_shares_immutable_texel_storage() {
    let cubemap = SourceCubemapMipChain::new(
        1,
        1,
        vec![[0.25, 0.5, 0.75, 1.0]; SOURCE_CUBEMAP_FACE_COUNT],
        1,
        1,
        vec![[0.5, 0.25, 0.75, 1.0]; SOURCE_CUBEMAP_FACE_COUNT],
    );
    let cloned = cubemap.clone();

    assert!(std::sync::Arc::ptr_eq(
        &cubemap.source_texels,
        &cloned.source_texels
    ));
    assert!(std::sync::Arc::ptr_eq(
        &cubemap.pmrem_texels,
        &cloned.pmrem_texels
    ));
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
fn source_cubemap_pmrem_layout_uses_independent_result_size_and_full_mip_chain() {
    let source_face_size = 4;
    let source_mip_count = source_cubemap_mip_count(source_face_size);
    let source_texels = vec![
        [0.25, 0.5, 0.75, 1.0];
        source_cubemap_sample_count(source_face_size, source_mip_count)
    ];
    let source_cubemap = SourceCubemapMipChain::new(
        source_face_size,
        source_mip_count,
        source_texels,
        1,
        1,
        vec![[0.25, 0.5, 0.75, 1.0]; SOURCE_CUBEMAP_FACE_COUNT],
    );
    let pmrem_layout = SourceCubemapPmremLayout::from_face_size(32);
    let cubemap = source_cubemap.with_pmrem_face_size(
        pmrem_layout.face_size(),
        SourceCubemapPrefilterQuality::Fast,
    );

    assert_eq!(pmrem_layout.face_size(), 32);
    assert_eq!(pmrem_layout.mip_count(), 6);
    assert_eq!(cubemap.source_face_size(), source_face_size);
    assert_eq!(cubemap.source_mip_count(), source_mip_count);
    assert_eq!(cubemap.pmrem_face_size(), 32);
    assert_eq!(cubemap.pmrem_mip_count(), 6);
    for texel in cubemap.pmrem_texels() {
        assert_vec4_close(*texel, [0.25, 0.5, 0.75, 1.0]);
    }
}

#[test]
fn source_cubemap_constructor_clamps_layouts_to_available_full_mip_chains() {
    let source_face_size = 4;
    let source_mip_count = source_cubemap_mip_count(source_face_size);
    let pmrem_face_size = 8;
    let pmrem_mip_count = source_cubemap_mip_count(pmrem_face_size);
    let cubemap = SourceCubemapMipChain::new(
        source_face_size,
        source_mip_count + 1,
        vec![
            [0.25, 0.5, 0.75, 1.0];
            source_cubemap_sample_count(source_face_size, source_mip_count)
        ],
        pmrem_face_size,
        pmrem_mip_count + 1,
        vec![[0.25, 0.5, 0.75, 1.0]; source_cubemap_sample_count(pmrem_face_size, pmrem_mip_count)],
    );

    assert_eq!(cubemap.source_mip_count(), source_mip_count);
    assert_eq!(cubemap.pmrem_mip_count(), pmrem_mip_count);
}

#[test]
fn source_cubemap_external_mip_builder_clamps_before_validating_storage() {
    let face_size = 4;
    let full_mip_count = source_cubemap_mip_count(face_size);
    let cubemap = build_source_cubemap_from_source_mips(
        face_size,
        full_mip_count + 2,
        vec![[0.25, 0.5, 0.75, 1.0]; source_cubemap_sample_count(face_size, full_mip_count)],
    );

    assert_eq!(cubemap.source_mip_count(), full_mip_count);
}

#[test]
fn source_cubemap_pmrem_texel_clamps_mip_before_resolving_truncated_mip_size() {
    let source_face_size = 1;
    let pmrem_face_size = 8;
    let pmrem_mip_count = 2;
    let mut pmrem_texels =
        vec![[0.0; 4]; source_cubemap_sample_count(pmrem_face_size, pmrem_mip_count)];
    let expected = [0.25, 0.5, 0.75, 1.0];
    let last_mip_size = source_cubemap_mip_size(pmrem_face_size, pmrem_mip_count - 1);
    let last_mip_offset = source_cubemap_face_mip_offset(
        pmrem_face_size,
        pmrem_mip_count,
        CubemapFace::PositiveX,
        pmrem_mip_count - 1,
    );
    pmrem_texels[last_mip_offset + 3 * last_mip_size as usize + 3] = expected;
    let cubemap = SourceCubemapMipChain::new(
        source_face_size,
        1,
        vec![[0.0; 4]; SOURCE_CUBEMAP_FACE_COUNT],
        pmrem_face_size,
        pmrem_mip_count,
        pmrem_texels,
    );

    assert_eq!(
        cubemap.pmrem_texel(CubemapFace::PositiveX, u32::MAX, 3, 3),
        expected
    );
}

#[test]
fn source_cubemap_roughness_mip_mapping_matches_shader_contract() {
    let mip_count = SOURCE_CUBEMAP_PMREM_MIP_COUNT;
    assert_close(source_cubemap_pmrem_mip_from_roughness(0.0, mip_count), 0.0);
    assert_close(
        source_cubemap_pmrem_mip_from_roughness(1.0, mip_count),
        (mip_count - 2) as Real,
    );
    assert_close(source_cubemap_roughness_from_pmrem_mip(0, mip_count), 0.0);
    assert!(source_cubemap_roughness_from_pmrem_mip(mip_count - 3, mip_count) < 1.0);
    assert_close(
        source_cubemap_roughness_from_pmrem_mip(mip_count - 2, mip_count),
        1.0,
    );
    assert_close(
        source_cubemap_roughness_from_pmrem_mip(mip_count - 1, mip_count),
        1.0,
    );

    let mut previous = 0.0;
    for mip in 1..(mip_count - 1) {
        let roughness = source_cubemap_roughness_from_pmrem_mip(mip, mip_count);
        assert!(
            roughness >= previous,
            "roughness should increase with mip level, mip={mip} roughness={roughness} previous={previous}"
        );
        assert_close(
            source_cubemap_pmrem_mip_from_roughness(roughness, mip_count),
            mip as Real,
        );
        previous = roughness;
    }
}

#[test]
fn source_cubemap_pmrem_wgsl_lookup_matches_public_roughness_contract() {
    const ENVIRONMENT_WGSL: &str =
        include_str!("../../../../../graphics/shader/wgsl/zr_environment.wgsl");
    let expected = format!(
        "return clamp(max_mip - {SOURCE_CUBEMAP_ROUGHEST_MIP:.1} + {SOURCE_CUBEMAP_ROUGHNESS_MIP_SCALE:.1} * log2(clamped_roughness), 0.0, max_mip);"
    );

    assert!(
        ENVIRONMENT_WGSL.contains(&expected),
        "environment WGSL must use the source-cubemap PMREM roughness constants"
    );
}

#[test]
fn source_cubemap_public_roughness_constants_match_plan06_ue_mapping() {
    assert_close(SOURCE_CUBEMAP_ROUGHEST_MIP, 1.0);
    assert_close(SOURCE_CUBEMAP_ROUGHNESS_MIP_SCALE, 1.2);
}

#[test]
fn source_cubemap_irradiance_mip_prefers_thirty_two_face_source() {
    assert_eq!(source_cubemap_irradiance_mip_level(16, 5), 0);
    assert_eq!(source_cubemap_irradiance_mip_level(64, 7), 1);
    assert_eq!(source_cubemap_irradiance_mip_level(256, 9), 3);
}

#[test]
fn source_cubemap_angular_filter_selects_higher_resolution_input_mips() {
    let mip_count = source_cubemap_mip_count(512);
    let input_mips = (1..mip_count)
        .map(|mip| {
            let mip_size = source_cubemap_mip_size(512, mip);
            mipmap::source_cubemap_angular_input_mip(
                mip_count,
                mipmap::source_cubemap_angular_cone_angle(mip_size),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(input_mips, vec![0, 0, 0, 1, 2, 3, 4, 5, 6]);
    assert!(
        input_mips
            .iter()
            .enumerate()
            .all(|(index, input_mip)| *input_mip < index as u32 + 1),
        "Unreal angular filtering should read a quality-biased higher-resolution average mip"
    );
}

#[test]
fn source_cubemap_average_mips_cover_non_power_of_two_edges() {
    let face_size = 5;
    let mip_count = source_cubemap_mip_count(face_size);
    let mut base_storage = vec![[0.0; 4]; source_cubemap_sample_count(face_size, mip_count)];
    let face = CubemapFace::PositiveX;
    let offset = source_cubemap_face_mip_offset(face_size, mip_count, face, 0);
    for y in 0..face_size {
        for x in 0..face_size {
            let value = (y * face_size + x) as Real;
            base_storage[offset + y as usize * face_size as usize + x as usize] =
                [value, value, value, 1.0];
        }
    }

    let average_mips =
        mipmap::source_cubemap_average_mips_from_base(&base_storage, face_size, mip_count);
    let edge = source_storage_texel_at(&average_mips, face_size, mip_count, face, 1, 1, 1);
    let expected = (12.0 + 13.0 + 14.0 + 17.0 + 18.0 + 19.0 + 22.0 + 23.0 + 24.0) / 9.0;

    assert_vec4_close(edge, [expected, expected, expected, 1.0]);
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

    assert_eq!(cubemap.source_face_size(), face_size);
    assert_eq!(cubemap.source_mip_count(), 3);
    for face in CubemapFace::ALL {
        let expected = (face.index() + 1) as Real;
        assert_vec4_close(
            cubemap.pmrem_texel(face, 0, 64, 32),
            [expected, expected * 0.5, expected * 0.25, 1.0],
        );
    }
    assert!(cubemap
        .pmrem_texels()
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
fn source_cubemap_ggx_pmrem_mips_blur_high_frequency_source() {
    let cubemap = build_source_cubemap_from_equirect(8, |u, _| {
        if u < 0.5 {
            [0.0, 0.0, 0.0, 1.0]
        } else {
            [1.0, 1.0, 1.0, 1.0]
        }
    });
    let last_mip = cubemap.pmrem_mip_count() - 1;
    let last = cubemap.pmrem_texel(CubemapFace::PositiveX, last_mip, 0, 0);

    assert_eq!(
        cubemap.pmrem_texel(CubemapFace::NegativeX, 0, 64, 64)[0],
        0.0
    );
    assert_eq!(
        cubemap.pmrem_texel(CubemapFace::PositiveX, 0, 64, 64)[0],
        1.0
    );
    assert!(
        last[0] > 0.1 && last[0] < 0.9,
        "lowest radiance mip should be blurred toward the environment average, got {last:?}"
    );
    for face in CubemapFace::ALL {
        assert_eq!(cubemap.pmrem_texel(face, last_mip, 0, 0), last);
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
        source_cubemap_pmrem_mip_from_roughness(1.0, cubemap.pmrem_mip_count()).round() as u32;
    assert!(
        saturated_mip > 0,
        "roughness=1 should select a PMREM mip below the base level"
    );
    let mip_size = source_cubemap_mip_size(cubemap.pmrem_face_size(), saturated_mip);
    let previous_variance = mip_luma_variance(&cubemap, saturated_mip - 1);
    let saturated_variance = mip_luma_variance(&cubemap, saturated_mip);
    let mut max_downsample_luma_delta: Real = 0.0;

    for face in CubemapFace::ALL {
        for y in 0..mip_size {
            for x in 0..mip_size {
                let direction = cubemap_texel_direction(face, x, y, mip_size);
                let ordinary_downsample = sample_cubemap_linear_at_mip(
                    cubemap.pmrem_texels(),
                    cubemap.pmrem_face_size(),
                    cubemap.pmrem_mip_count(),
                    direction,
                    saturated_mip - 1,
                );
                let actual = cubemap.pmrem_texel(face, saturated_mip, x, y);
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
fn source_cubemap_ggx_pmrem_reduces_mip_luma_variance() {
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

    assert!(
        rough_variance < base_variance * 0.45,
        "rough PMREM mip should reduce high-frequency luma variance, base={base_variance} rough={rough_variance}"
    );
}

#[test]
fn source_cubemap_angular_mip_matches_unreal_cone_filter_reference() {
    let face_size = 8;
    let mip_count = source_cubemap_mip_count(face_size);
    let face_texel_count = face_size as usize * face_size as usize;
    let mut captured = Vec::with_capacity(face_texel_count * SOURCE_CUBEMAP_FACE_COUNT);
    for face in CubemapFace::ALL {
        for y in 0..face_size {
            for x in 0..face_size {
                let index =
                    face.index() * face_texel_count + y as usize * face_size as usize + x as usize;
                let value =
                    (index + 1) as Real / (face_texel_count * SOURCE_CUBEMAP_FACE_COUNT) as Real;
                captured.push([value, value * value, 1.0 - value, 1.0]);
            }
        }
    }

    let source_storage = source_storage_from_captured_faces(face_size, mip_count, &captured);
    let mip_level = 1;
    let mip_size = source_cubemap_mip_size(face_size, mip_level);
    for (face, x, y) in [
        (CubemapFace::PositiveX, 0, 1),
        (CubemapFace::PositiveY, 2, 0),
        (CubemapFace::NegativeZ, 3, 3),
    ] {
        let expected = unreal_angular_filter_reference_texel(
            &captured, face_size, mip_count, face, mip_level, x, y,
        );
        let actual =
            source_storage_texel_at(&source_storage, face_size, mip_count, face, mip_level, x, y);
        assert_vec4_close_with_tolerance(actual, expected, 0.000_01);
        assert!(x < mip_size && y < mip_size);
    }
}

#[test]
fn source_cubemap_angular_high_mip_matches_selected_average_mip_reference() {
    let face_size = 32;
    let mip_count = source_cubemap_mip_count(face_size);
    let face_texel_count = face_size as usize * face_size as usize;
    let mut captured = Vec::with_capacity(face_texel_count * SOURCE_CUBEMAP_FACE_COUNT);
    for face in CubemapFace::ALL {
        for y in 0..face_size {
            for x in 0..face_size {
                let direction = cubemap_texel_direction(face, x, y, face_size);
                let value = 0.4 + direction[0] * 0.2 + direction[1] * 0.1 + direction[2] * 0.3;
                captured.push([value, value * value, 1.0 - value, 0.25 + value * 0.5]);
            }
        }
    }

    let base_storage = source_base_storage_from_captured_faces(face_size, mip_count, &captured);
    let average_mips =
        mipmap::source_cubemap_average_mips_from_base(&base_storage, face_size, mip_count);
    let source_storage = mipmap::source_cubemap_mips_from_base(&base_storage, face_size, mip_count);
    let mip_level = 4;
    let mip_size = source_cubemap_mip_size(face_size, mip_level);
    let cone_angle = mipmap::source_cubemap_angular_cone_angle(mip_size);
    let input_mip = mipmap::source_cubemap_angular_input_mip(mip_count, cone_angle);
    assert_eq!(
        input_mip, 1,
        "this guard must exercise the average input pyramid"
    );

    let expected = plan06_angular_filter_reference_texel(
        &average_mips,
        face_size,
        mip_count,
        input_mip,
        CubemapFace::NegativeY,
        mip_level,
        1,
        0,
    );
    let actual = source_storage_texel_at(
        &source_storage,
        face_size,
        mip_count,
        CubemapFace::NegativeY,
        mip_level,
        1,
        0,
    );
    assert_vec4_close_with_tolerance(actual, expected, 0.000_01);
}

fn source_storage_from_captured_faces(
    face_size: u32,
    mip_count: u32,
    captured: &[[Real; 4]],
) -> Vec<[Real; 4]> {
    let base_storage = source_base_storage_from_captured_faces(face_size, mip_count, captured);
    mipmap::source_cubemap_mips_from_base(&base_storage, face_size, mip_count)
}

fn source_base_storage_from_captured_faces(
    face_size: u32,
    mip_count: u32,
    captured: &[[Real; 4]],
) -> Vec<[Real; 4]> {
    let face_texel_count = face_size as usize * face_size as usize;
    let mut base_storage = vec![[0.0; 4]; source_cubemap_sample_count(face_size, mip_count)];
    for face in CubemapFace::ALL {
        let source_offset = face.index() * face_texel_count;
        let target_offset = source_cubemap_face_mip_offset(face_size, mip_count, face, 0);
        base_storage[target_offset..target_offset + face_texel_count]
            .copy_from_slice(&captured[source_offset..source_offset + face_texel_count]);
    }
    base_storage
}

fn source_storage_texel_at(
    texels: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
    face: CubemapFace,
    mip_level: u32,
    x: u32,
    y: u32,
) -> [Real; 4] {
    let mip_size = source_cubemap_mip_size(face_size, mip_level);
    let offset = source_cubemap_face_mip_offset(face_size, mip_count, face, mip_level);
    texels[offset + y as usize * mip_size as usize + x as usize]
}

fn unreal_angular_filter_reference_texel(
    base_texels: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
    output_face: CubemapFace,
    mip_level: u32,
    x: u32,
    y: u32,
) -> [Real; 4] {
    assert_eq!(
        mip_level, 1,
        "the golden reference intentionally reads mip zero"
    );
    assert_eq!(mip_count, source_cubemap_mip_count(face_size));
    let input_texels = source_base_storage_from_captured_faces(face_size, mip_count, base_texels);
    plan06_angular_filter_reference_texel(
        &input_texels,
        face_size,
        mip_count,
        0,
        output_face,
        mip_level,
        x,
        y,
    )
}

fn plan06_angular_filter_reference_texel(
    input_texels: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
    input_mip: u32,
    output_face: CubemapFace,
    mip_level: u32,
    x: u32,
    y: u32,
) -> [Real; 4] {
    let mip_size = source_cubemap_mip_size(face_size, mip_level);
    let input_size = source_cubemap_mip_size(face_size, input_mip);
    let filter_direction = cubemap_texel_direction(output_face, x, y, mip_size);
    let cone_angle =
        (std::f32::consts::FRAC_PI_2 / mip_size as Real).clamp(0.002, std::f32::consts::FRAC_PI_2);
    let direction_threshold = cone_angle.cos().min(0.9999);
    let inverse_kernel_width = 1.0 / (1.0 - direction_threshold);
    let mut color = [0.0; 4];
    let mut weight_sum = 0.0;

    for face in CubemapFace::ALL {
        for source_y in 0..input_size {
            for source_x in 0..input_size {
                let source_direction =
                    cubemap_texel_direction(face, source_x, source_y, input_size);
                let direction_dot = dot3_test(filter_direction, source_direction);
                if direction_dot <= direction_threshold {
                    continue;
                }
                let kernel = (1.0 - (1.0 - direction_dot) * inverse_kernel_width).clamp(0.0, 1.0);
                let kernel = kernel * kernel * (3.0 - 2.0 * kernel);
                let weight = kernel * cubemap_texel_solid_angle(source_x, source_y, input_size);
                let texel = source_storage_texel_at(
                    input_texels,
                    face_size,
                    mip_count,
                    face,
                    input_mip,
                    source_x,
                    source_y,
                );
                for component in 0..4 {
                    color[component] += texel[component] * weight;
                }
                weight_sum += weight;
            }
        }
    }

    assert!(weight_sum > Real::EPSILON);
    for component in &mut color {
        *component /= weight_sum;
    }
    color
}

fn dot3_test(first: [Real; 3], second: [Real; 3]) -> Real {
    first[0] * second[0] + first[1] * second[1] + first[2] * second[2]
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
    assert_vec4_close_with_tolerance(actual, expected, 0.00001);
}

fn assert_vec4_close_with_tolerance(actual: [Real; 4], expected: [Real; 4], tolerance: Real) {
    for index in 0..4 {
        assert!(
            (actual[index] - expected[index]).abs() <= tolerance,
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
    let mip_size = source_cubemap_mip_size(cubemap.pmrem_face_size(), mip_level);
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    let mut count = 0.0;
    for face in CubemapFace::ALL {
        for y in 0..mip_size {
            for x in 0..mip_size {
                let texel = cubemap.pmrem_texel(face, mip_level, x, y);
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
