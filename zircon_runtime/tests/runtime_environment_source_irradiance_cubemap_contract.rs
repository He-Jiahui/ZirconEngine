use zircon_runtime::core::framework::render::{
    build_source_cubemap_from_equirect, build_source_cubemap_irradiance_cube,
    source_cubemap_evaluate_irradiance_sh9, source_cubemap_sample_irradiance_cube, CubemapFace,
    SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
};

#[test]
fn runtime_environment_source_irradiance_cubemap_preserves_constant_diffuse_environment() {
    let cubemap = build_source_cubemap_from_equirect(64, |_, _| [0.25, 0.5, 0.75, 1.0]);

    let irradiance = build_source_cubemap_irradiance_cube(&cubemap);

    assert_eq!(
        irradiance.face_size(),
        SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE
    );
    for face in CubemapFace::ALL {
        for y in 0..irradiance.face_size() {
            for x in 0..irradiance.face_size() {
                assert_rgb_close(irradiance.texel(face, x, y), [0.25, 0.5, 0.75], 0.002);
            }
        }
    }
}

#[test]
fn runtime_environment_source_irradiance_cubemap_matches_sh9_for_low_frequency_environment() {
    let cubemap = build_source_cubemap_from_equirect(64, |_, v| {
        let sky_weight = 1.0 - v;
        [sky_weight, sky_weight * 0.75, sky_weight * 0.5, 1.0]
    });
    let irradiance = build_source_cubemap_irradiance_cube(&cubemap);

    for direction in low_frequency_probe_directions() {
        let iem = source_cubemap_sample_irradiance_cube(&irradiance, direction);
        let sh9 = source_cubemap_evaluate_irradiance_sh9(cubemap.irradiance_sh9(), direction);

        assert_rgb_close(iem, sh9, 0.055);
    }
}

fn low_frequency_probe_directions() -> impl Iterator<Item = [f32; 3]> {
    let mut directions = Vec::new();
    for y in [-0.75_f32, -0.25, 0.25, 0.75] {
        let radius = (1.0 - y * y).sqrt();
        for angle_index in 0..8 {
            let angle = std::f32::consts::TAU * angle_index as f32 / 8.0;
            directions.push([radius * angle.cos(), y, radius * angle.sin()]);
        }
    }
    directions.into_iter()
}

fn assert_rgb_close(actual: [f32; 3], expected: [f32; 3], tolerance: f32) {
    for index in 0..3 {
        assert!(
            (actual[index] - expected[index]).abs() <= tolerance,
            "component {index}: actual={actual:?} expected={expected:?} tolerance={tolerance}"
        );
    }
}
