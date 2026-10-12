use super::{sky_shader_source, SKY_SHADER};

#[test]
fn skybox_shader_variant_removes_volumetric_bindings_when_disabled() {
    let disabled = sky_shader_source(false);
    let enabled = sky_shader_source(true);

    for binding in ["@binding(25)", "@binding(26)", "@binding(27)"] {
        assert!(!disabled.contains(binding));
        assert!(enabled.contains(binding));
    }
}

#[test]
fn skybox_shader_reconstructs_camera_ray_before_source_cubemap_sampling() {
    for expected in [
        "fn skybox_world_direction_from_ndc",
        "scene.inverse_view_proj",
        // BUG: [CR-R02-runtime_wave12_graphics_renderer_overlay-0002] 运行此测试时第三项文本断言必失败：完整拼接 WGSL 已采用正交方向提前返回，不含这里要求的旧相机基向量组合；前两项已命中。
        "camera_forward + ndc.x * camera_right + ndc.y * camera_up",
        "let direction = skybox_world_direction_from_ndc(ndc);",
        "fn skybox_fix_cube_lookup",
        "skybox_fix_cube_lookup(rotated, 0.0)",
    ] {
        assert!(
            SKY_SHADER.contains(expected),
            "skybox shader should use `{expected}` for camera-space cubemap lookup"
        );
    }
}

#[test]
fn skybox_orthographic_direction_is_parallel_and_skips_inverse_projection() {
    let direction = SKY_SHADER
        .split("fn skybox_world_direction_from_ndc(")
        .nth(1)
        .and_then(|source| source.split("fn source_cubemap_sky_color(").next())
        .expect("skybox shader should retain its camera-ray owner");
    let orthographic_endpoint = direction
        .find("if (camera_direction_weight >= 1.0) {")
        .expect("orthographic skybox rays must return before inverse projection");
    let inverse_projection = direction
        .find("scene.inverse_view_proj * vec4<f32>(ndc.x, ndc.y, 1.0, 1.0)")
        .expect("perspective skybox rays must retain inverse projection");

    assert!(orthographic_endpoint < inverse_projection);
    assert!(direction.contains("-scene.camera_view_direction.xyz"));
    assert!(!direction.contains("right_far_world"));
    assert!(!direction.contains("up_far_world"));
    assert!(!direction.contains("camera_forward + ndc.x * camera_right + ndc.y * camera_up"));
}

#[test]
fn skybox_shader_rotation_uses_cpu_precomputed_trigonometry() {
    let rotation = SKY_SHADER
        .split("fn skybox_rotated_direction_normalized(")
        .nth(1)
        .and_then(|source| source.split("fn skybox_normalize_or_fallback(").next())
        .expect("skybox shader should retain its rotation helper");

    let environment_sample_params = SKY_SHADER
        .find("environment_sample_params: vec4<f32>,")
        .expect("skybox SceneUniform should retain the environment sampling parameters");
    let rotation_tail = SKY_SHADER
        .find("environment_rotation_sin_cos: vec4<f32>,")
        .expect("skybox SceneUniform should append the rotation tail");
    assert!(
        environment_sample_params < rotation_tail,
        "the skybox SceneUniform mirror must append the rotation tail after existing fields"
    );
    assert!(rotation.contains("scene.environment_rotation_sin_cos.z < 0.5"));
    assert!(
        rotation.contains("scene.environment_rotation_sin_cos.x")
            && rotation.contains("scene.environment_rotation_sin_cos.y")
    );
    assert!(!rotation.contains("sin(rotation)"));
    assert!(!rotation.contains("cos(rotation)"));
}

#[test]
fn skybox_shader_feature_variants_are_valid_wgsl() {
    for (label, source) in [
        ("disabled", sky_shader_source(false)),
        ("enabled", sky_shader_source(true)),
    ] {
        let module = naga::front::wgsl::parse_str(&source)
            .unwrap_or_else(|error| panic!("{label}: {}", error.emit_to_string(&source)));
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );

        validator
            .validate(&module)
            .unwrap_or_else(|error| panic!("{label} skybox shader should validate: {error}"));
    }
}

#[test]
fn skybox_shader_applies_integrated_volumetric_lighting_at_far_depth() {
    for expected in [
        "output.clip_position = vec4<f32>(position, 1.0, 1.0);",
        "@group(1) @binding(25) var<uniform> zr_volumetric_apply_params",
        "@group(1) @binding(26) var zr_volumetric_integrated: texture_3d<f32>;",
        "@group(1) @binding(27) var zr_volumetric_sampler: sampler;",
        "zr_volumetric_apply(color, input.clip_position.xy, 1.0)",
    ] {
        assert!(
            SKY_SHADER.contains(expected),
            "skybox shader should use volumetric contract `{expected}`"
        );
    }
}
