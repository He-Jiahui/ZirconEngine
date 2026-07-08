use zircon_runtime::core::framework::render::{
    build_environment_brdf_lut, environment_brdf_lut_integrate, environment_brdf_lut_texel_index,
};

#[test]
fn runtime_environment_brdf_lut_corner_values_match_split_sum_contract() {
    let sharp_normal = environment_brdf_lut_integrate(1.0, 0.0, 1024);
    assert!(
        sharp_normal[0] > 0.99,
        "normal-incidence smooth dielectric scale should approach 1, got {sharp_normal:?}"
    );
    assert!(
        sharp_normal[1] < 0.01,
        "normal-incidence smooth dielectric bias should approach 0, got {sharp_normal:?}"
    );

    let rough_normal = environment_brdf_lut_integrate(1.0, 1.0, 1024);
    assert!(
        rough_normal[0] + rough_normal[1] < 0.5,
        "fully rough normal-incidence split-sum response should be attenuated, got {rough_normal:?}"
    );
}

#[test]
fn runtime_environment_brdf_lut_keeps_perfect_mirror_grazing_energy_conserved() {
    for no_v in [0.001, 0.005, 0.01, 0.05, 0.1] {
        let [scale, bias] = environment_brdf_lut_integrate(no_v, 0.0, 4096);
        let perfect_mirror_response = scale + bias;

        assert!(
            perfect_mirror_response <= 1.01,
            "smooth F0=1 mirror response should not amplify grazing IBL energy, no_v={no_v}, scale={scale}, bias={bias}, response={perfect_mirror_response}"
        );
    }
}

#[test]
fn runtime_environment_brdf_lut_builder_outputs_finite_nonnegative_texels() {
    let size = 8;
    let texels = build_environment_brdf_lut(size, 64);

    assert_eq!(texels.len(), size as usize * size as usize);
    assert_eq!(environment_brdf_lut_texel_index(size, 0, 0), 0);
    assert_eq!(
        environment_brdf_lut_texel_index(size, size + 16, size + 16),
        texels.len() - 1
    );

    for texel in texels {
        assert!(texel[0].is_finite(), "{texel:?}");
        assert!(texel[1].is_finite(), "{texel:?}");
        assert!(texel[0] >= 0.0, "{texel:?}");
        assert!(texel[1] >= 0.0, "{texel:?}");
    }
}
