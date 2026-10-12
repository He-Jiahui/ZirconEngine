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
fn environment_brdf_lut_matches_unreal_joint_smith_reference_anchors() {
    for (no_v, roughness, expected) in [
        (0.1, 1.0, [0.754189, 0.025194]),
        (0.5, 0.6, [0.671656, 0.013660]),
    ] {
        let actual =
            environment_brdf_lut_integrate(no_v, roughness, ENVIRONMENT_BRDF_LUT_SAMPLE_COUNT);
        for channel in 0..2 {
            assert!(
                (actual[channel] - expected[channel]).abs() <= 0.001,
                "NoV={no_v}, roughness={roughness}, actual={actual:?}, expected={expected:?}"
            );
        }
    }
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
            let runtime =
                environment_brdf_lut_integrate(no_v, roughness, ENVIRONMENT_BRDF_LUT_SAMPLE_COUNT);
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
    assert!(mean_error <= 0.0031, "mean absolute error={mean_error}");
    assert!(
        maximum_error <= 0.023,
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
