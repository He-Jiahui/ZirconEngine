const ENVIRONMENT_SHADER: &str = include_str!("../src/graphics/shader/wgsl/zr_environment.wgsl");
const SKYBOX_SHADER: &str =
    include_str!("../src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl");
const REALTIME_CAPTURE_SHADER: &str = include_str!(
    "../src/graphics/scene/scene_renderer/environment/shaders/realtime_ibl_capture.wgsl"
);
const SKYBOX_SETTINGS_SOURCE: &str =
    include_str!("../src/core/framework/render/environment/skybox.rs");

fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing {signature}"));
    let function = &source[start..];
    let body_start = function
        .find('{')
        .unwrap_or_else(|| panic!("missing opening brace for {signature}"));
    let body = &function[body_start + 1..];
    let body_end = body
        .find('}')
        .unwrap_or_else(|| panic!("missing closing brace for {signature}"));
    &body[..body_end]
}

#[test]
fn runtime_environment_wgpu_cubemap_sampling_does_not_warp_lookup_directions() {
    for (label, source, signature) in [
        (
            "environment",
            ENVIRONMENT_SHADER,
            "fn zr_environment_fix_cube_lookup_for_face_size",
        ),
        ("skybox", SKYBOX_SHADER, "fn skybox_fix_cube_lookup"),
    ] {
        let body = function_body(source, signature);
        assert!(
            body.contains("return direction;"),
            "{label} must preserve the cube lookup direction"
        );
        assert!(
            !body.contains("adjusted"),
            "{label} must not retain legacy edge-warp state"
        );
        assert!(
            !body.contains("exp2("),
            "{label} must not retain legacy LOD edge warping"
        );
    }
}

#[test]
fn runtime_environment_zero_rotation_skips_trigonometry() {
    let rotation = ENVIRONMENT_SHADER
        .split("fn zr_environment_rotated_direction(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn zr_environment_fix_cube_lookup_for_face_size(")
                .next()
        })
        .expect("environment shader should retain the shared rotation owner");

    let zero_guard = rotation
        .find("if (rotation == 0.0)")
        .expect("the default zero rotation must retain a uniform fast path");
    let identity_return = rotation[zero_guard..]
        .find("return direction;")
        .map(|offset| zero_guard + offset)
        .expect("zero rotation must preserve the already-normalized direction");
    let sine = rotation
        .find("let s = sin(rotation);")
        .expect("nonzero rotation must retain its sine");
    let cosine = rotation
        .find("let c = cos(rotation);")
        .expect("nonzero rotation must retain its cosine");

    assert!(
        zero_guard < identity_return && identity_return < sine && sine < cosine,
        "zero rotation must return before evaluating per-pixel trigonometry"
    );
}

#[test]
fn runtime_environment_cpu_sun_rotation_is_inverse_of_shader_lookup_rotation() {
    let environment_rotation = ENVIRONMENT_SHADER
        .split("fn zr_environment_rotated_direction(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn zr_environment_fix_cube_lookup_for_face_size(")
                .next()
        })
        .expect("environment shader should retain the shared rotation owner");
    let skybox_rotation = SKYBOX_SHADER
        .split("fn skybox_rotated_direction_normalized(")
        .nth(1)
        .and_then(|source| source.split("fn skybox_normalize_or_fallback(").next())
        .expect("skybox shader should retain its rotation owner");
    for (label, rotation) in [
        ("environment", environment_rotation),
        ("skybox", skybox_rotation),
    ] {
        assert!(
            rotation.contains("direction.x * c - direction.z * s"),
            "{label} lookup rotation must retain its positive-angle x row"
        );
        assert!(
            rotation.contains("direction.x * s + direction.z * c"),
            "{label} lookup rotation must retain its positive-angle z row"
        );
    }

    let cpu_rotation = SKYBOX_SETTINGS_SOURCE
        .split("fn direction_for_sampling_rotation(")
        .nth(1)
        .and_then(|source| source.split("impl ProceduralSkyParams").next())
        .expect("procedural sky should retain the CPU sampling-rotation owner");
    assert!(cpu_rotation.contains("self.direction.x * cosine + self.direction.z * sine"));
    assert!(cpu_rotation.contains("-self.direction.x * sine + self.direction.z * cosine"));
}

#[test]
fn runtime_environment_skybox_reuses_reconstructed_normalized_direction() {
    let rotation = SKYBOX_SHADER
        .split("fn skybox_rotated_direction_normalized(")
        .nth(1)
        .and_then(|source| source.split("fn skybox_normalize_or_fallback(").next())
        .expect("skybox shader should retain the normalized rotation owner");
    assert!(
        !rotation.contains("normalize("),
        "the skybox rotation owner must not renormalize its reconstructed unit direction"
    );
    let zero_guard = rotation
        .find("if (rotation == 0.0)")
        .expect("the default skybox rotation must retain a uniform fast path");
    let identity_return = rotation[zero_guard..]
        .find("return direction;")
        .map(|offset| zero_guard + offset)
        .expect("zero skybox rotation must preserve the reconstructed direction");
    let sine = rotation
        .find("let s = sin(rotation);")
        .expect("nonzero skybox rotation must retain its sine");
    assert!(
        zero_guard < identity_return && identity_return < sine,
        "zero skybox rotation must return before trigonometry"
    );

    let source_sample = SKYBOX_SHADER
        .split("fn source_cubemap_sky_color(")
        .nth(1)
        .and_then(|source| source.split("fn procedural_sun_radiance(").next())
        .expect("skybox shader should retain source cubemap sampling");
    assert!(
        source_sample.contains("skybox_rotated_direction_normalized(direction)"),
        "source cubemap sky sampling must use the normalized rotation owner"
    );
}

#[test]
fn runtime_environment_source_cubemap_reflections_use_pmrem_before_procedural_fallback() {
    let reflection = ENVIRONMENT_SHADER
        .split("fn zr_environment_sky_reflection_color(")
        .nth(1)
        .and_then(|source| source.split("fn zr_environment_planar_reflection(").next())
        .expect("environment shader should retain the sky-reflection owner");

    for expected in [
        "if (zr_environment_is_source_cubemap() || zr_environment_is_realtime_ibl())",
        "let lod = zr_environment_mip_from_roughness(roughness, max_mip);",
        "return zr_environment_specular_pmrem_color_at_clamped_lod_normalized(reflected, lod);",
    ] {
        assert!(
            reflection.contains(expected),
            "source/realtime IBL reflection must use PMREM through `{expected}`"
        );
    }
    let fallback = reflection
        .split(
            "return zr_environment_specular_pmrem_color_at_clamped_lod_normalized(reflected, lod);",
        )
        .nth(1)
        .expect("procedural fallback should remain separate from source/realtime IBL");
    assert!(
        fallback.contains("return zr_environment_procedural_sky_color_normalized(reflected);"),
        "a procedural fallback without PMREM must preserve the reflected direction"
    );
    assert_eq!(
        fallback
            .matches("zr_environment_procedural_sky_color_normalized(")
            .count(),
        1,
        "the fallback must contain exactly one sky lookup"
    );
    assert_eq!(
        fallback
            .matches("zr_environment_procedural_sky_color_normalized(reflected)")
            .count(),
        1,
        "the sole fallback sky lookup must preserve the reflected direction"
    );
    for forbidden in [
        "zr_environment_procedural_sky_color_normalized(normal)",
        "mix(sharp_reflection, rough_reflection, roughness)",
    ] {
        assert!(
            !fallback.contains(forbidden),
            "a procedural fallback without PMREM must not synthesize roughness with `{forbidden}`"
        );
    }
}

#[test]
fn runtime_environment_procedural_pbr_reuses_normalized_directions() {
    let normalized_sky = ENVIRONMENT_SHADER
        .split("fn zr_environment_procedural_sky_color_normalized(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn zr_environment_procedural_sky_color(")
                .next()
        })
        .expect("environment shader should retain the normalized procedural-sky owner");
    assert!(
        !normalized_sky.contains("zr_environment_normalize_or_zero("),
        "the normalized procedural-sky path must not normalize its input a second time"
    );

    let defensive_sky = function_body(
        ENVIRONMENT_SHADER,
        "fn zr_environment_procedural_sky_color(",
    );
    assert!(
        defensive_sky.contains(
            "zr_environment_procedural_sky_color_normalized(\n        zr_environment_normalize_or_zero(direction),",
        ),
        "the public procedural-sky wrapper must retain defensive normalization"
    );

    let diffuse = ENVIRONMENT_SHADER
        .split("fn zr_environment_diffuse_color_normalized(")
        .nth(1)
        .and_then(|source| source.split("fn zr_environment_diffuse_color(").next())
        .expect("environment shader should retain the normalized diffuse owner");
    assert!(
        diffuse.contains("zr_environment_procedural_sky_color_normalized(normal)"),
        "procedural diffuse must reuse its normalized PBR normal"
    );
}

#[test]
fn runtime_environment_procedural_sun_uses_cpu_prepared_parameters() {
    let environment_sun = ENVIRONMENT_SHADER
        .split("fn zr_environment_procedural_sky_color_normalized(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn zr_environment_procedural_sky_color(")
                .next()
        })
        .expect("environment shader should retain procedural sky sampling");
    let skybox_sun = SKYBOX_SHADER
        .split("fn procedural_sun_radiance(")
        .nth(1)
        .and_then(|source| source.split("@fragment").next())
        .expect("skybox shader should retain procedural sun sampling");

    for (label, source, direction_owner) in [
        (
            "environment",
            environment_sun,
            "scene.sky_sun_direction.xyz",
        ),
        ("skybox", skybox_sun, "scene.sky_sun_direction.xyz"),
        (
            "realtime capture",
            REALTIME_CAPTURE_SHADER,
            "params.sun_direction.xyz",
        ),
    ] {
        assert!(
            source.contains(direction_owner),
            "{label} must consume the CPU-normalized sun direction"
        );
        assert!(
            !source.contains("sun_direction_length"),
            "{label} must not measure a uniform direction per invocation"
        );
        assert!(
            !source.contains("angular_radius"),
            "{label} must consume precomputed cosine edges"
        );
        assert!(
            !source.contains("cos("),
            "{label} must not compute sun cosine edges per invocation"
        );
    }
}

#[test]
fn runtime_environment_direct_procedural_sun_obeys_final_sampling_intensity() {
    let environment_sun = ENVIRONMENT_SHADER
        .split("fn zr_environment_procedural_sky_color_normalized(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn zr_environment_procedural_sky_color(")
                .next()
        })
        .expect("environment shader should retain procedural sky sampling");
    assert!(environment_sun.contains("return color * max(scene.environment_params.y, 0.0);"));

    let skybox_fragment = SKYBOX_SHADER
        .split("fn fs_main(")
        .nth(1)
        .expect("skybox shader should retain its fragment entry");
    assert!(skybox_fragment.contains(
        "color = (select(ground, sky, direction.y >= 0.0)\n            + procedural_sun_radiance(direction))\n            * intensity;"
    ));
}

#[test]
fn runtime_environment_pbr_pmrem_reuses_normalized_direction_and_clamped_lod() {
    let normalized_sample = function_body(
        ENVIRONMENT_SHADER,
        "fn zr_environment_specular_pmrem_color_at_clamped_lod_normalized",
    );
    assert!(
        normalized_sample.contains("zr_environment_rotated_direction(direction)"),
        "the normalized PMREM path must still apply runtime environment rotation"
    );
    for forbidden in [
        "zr_environment_normalize_or_zero(",
        "scene.environment_sample_params.w",
        "clamp(lod",
    ] {
        assert!(
            !normalized_sample.contains(forbidden),
            "the normalized PMREM path must reuse prepared input instead of `{forbidden}`"
        );
    }

    let defensive_sample = function_body(
        ENVIRONMENT_SHADER,
        "fn zr_environment_specular_pmrem_color_at_lod",
    );
    for expected in [
        "zr_environment_normalize_or_zero(direction)",
        "let clamped_lod = clamp(lod, 0.0, max_mip);",
        "zr_environment_specular_pmrem_color_at_clamped_lod_normalized(",
    ] {
        assert!(
            defensive_sample.contains(expected),
            "the public PMREM wrapper must retain defensive preparation through `{expected}`"
        );
    }
}

#[test]
fn runtime_environment_planar_reflection_short_circuits_pmrem_and_probe_work() {
    let reflection = ENVIRONMENT_SHADER
        .split("fn zr_environment_reflection_color_normalized(")
        .nth(1)
        .and_then(|source| source.split("fn zr_environment_diffuse_color(").next())
        .expect("environment shader should retain the reflection owner");

    let planar = reflection
        .find("let planar = zr_environment_planar_reflection(")
        .expect("reflection must evaluate the planar candidate");
    let planar_return = reflection[planar..]
        .find("return planar.rgb;")
        .map(|offset| planar + offset)
        .expect("a valid planar candidate must return before lower-priority reflection work");
    let reflected = reflection
        .find("let reflected = reflect(-view_dir, normal);")
        .expect("reflection must retain the reflected lookup direction");
    let sky = reflection
        .find("var sky = vec3<f32>(0.0);")
        .expect("the probe path must retain conditional sky storage");
    let probes = reflection
        .find("let selection = zr_environment_select_probes(")
        .expect("reflection must retain probe selection");
    let no_probes = reflection
        .find("if (zr_env_probe_header.probe_count == 0u)")
        .expect("reflection must skip probe selection when no probes exist");
    let no_probe_return = reflection[no_probes..]
        .find("return zr_environment_sky_reflection_color(reflected, clamped_roughness);")
        .map(|offset| no_probes + offset)
        .expect("the no-probe fast path must return sky reflection before probe selection");

    assert!(
        planar < planar_return
            && planar_return < reflected
            && reflected < no_probes
            && no_probes < no_probe_return
            && no_probe_return < probes
            && probes < sky,
        "planar reflection must short-circuit reflected-direction, PMREM sampling, and probe selection"
    );
}

#[test]
fn runtime_environment_full_probe_coverage_skips_zero_weight_sky_sample() {
    let reflection = ENVIRONMENT_SHADER
        .split("fn zr_environment_reflection_color_after_planar(")
        .nth(1)
        .and_then(|source| source.split("fn zr_environment_diffuse_color(").next())
        .expect("environment shader should retain the post-planar reflection owner");

    let no_probes = reflection
        .find("if (zr_env_probe_header.probe_count == 0u)")
        .expect("the no-probe sky fast path must remain explicit");
    let selection = reflection
        .find("let selection = zr_environment_select_probes(")
        .expect("reflection must select probes after the no-probe fast path");
    let sky_weight = reflection
        .find("let sky_weight = max(")
        .expect("reflection must retain explicit sky weighting");
    let sky_guard = reflection
        .find("if (sky_weight > 0.0)")
        .expect("zero-weight sky sampling must be guarded");
    let sky_sample = reflection[sky_guard..]
        .find("zr_environment_sky_reflection_color(")
        .map(|offset| sky_guard + offset)
        .expect("a positive sky weight must retain the sky sample");

    assert!(
        no_probes < selection
            && selection < sky_weight
            && sky_weight < sky_guard
            && sky_guard < sky_sample,
        "fully probe-covered pixels must not sample zero-weight sky PMREM"
    );
}

#[test]
fn runtime_environment_pbr_reuses_normalized_reflection_inputs() {
    let components = ENVIRONMENT_SHADER
        .split("fn zr_environment_pbr_components(")
        .nth(1)
        .and_then(|source| source.split("fn zr_environment_pbr_indirect(").next())
        .expect("environment shader should retain PBR indirect components");

    assert!(
        components.contains("zr_environment_reflection_color_normalized("),
        "PBR components must call the normalized reflection hot path"
    );
    assert!(
        !components.contains("zr_environment_reflection_color(\n"),
        "PBR components already normalize normal/view and clamp roughness before reflection"
    );

    let wrapper = ENVIRONMENT_SHADER
        .split("fn zr_environment_reflection_color(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn zr_environment_reflection_color_normalized(")
                .next()
        })
        .expect("environment shader should retain the defensive reflection wrapper");
    for expected in [
        "zr_environment_normalize_or_zero(normal_ws)",
        "zr_environment_normalize_or_zero(view_dir_ws)",
        "clamp(roughness, 0.0, 1.0)",
    ] {
        assert!(
            wrapper.contains(expected),
            "the defensive reflection wrapper must retain `{expected}`"
        );
    }

    let planar = wrapper
        .find("let planar = zr_environment_planar_reflection(")
        .expect("the defensive wrapper must evaluate planar reflection before normalizing inputs");
    let planar_return = wrapper[planar..]
        .find("return planar.rgb;")
        .map(|offset| planar + offset)
        .expect("a valid planar candidate must return from the defensive wrapper");
    let normal = wrapper
        .find("zr_environment_normalize_or_zero(normal_ws)")
        .expect("the wrapper must retain normal normalization for non-planar reflections");
    let view = wrapper
        .find("zr_environment_normalize_or_zero(view_dir_ws)")
        .expect("the wrapper must retain view normalization for non-planar reflections");
    assert!(
        planar < planar_return && planar_return < normal && normal < view,
        "planar reflection must avoid defensive normal/view normalization"
    );
}

#[test]
fn runtime_environment_full_metal_skips_zero_weight_diffuse_ibl() {
    let components = ENVIRONMENT_SHADER
        .split("fn zr_environment_pbr_components(")
        .nth(1)
        .and_then(|source| source.split("fn zr_environment_pbr_indirect(").next())
        .expect("environment shader should retain PBR indirect components");

    let diffuse_energy = components
        .find("let diffuse_energy_scale = 1.0 - clamped_metallic;")
        .expect("PBR indirect must retain metallic diffuse energy scaling");
    let guard = components
        .find("if (diffuse_energy_scale > 0.0)")
        .expect("zero-weight diffuse IBL must be guarded for full-metal materials");
    let diffuse_sample = components
        .find("zr_environment_diffuse_color_normalized(normal)")
        .expect("PBR indirect must retain diffuse IBL below the metallic guard");

    assert!(
        diffuse_energy < guard && guard < diffuse_sample,
        "full-metal materials must not evaluate diffuse IBL before its zero-weight guard"
    );
}

#[test]
fn runtime_environment_pbr_diffuse_reuses_its_normalized_normal() {
    let normalized_diffuse = ENVIRONMENT_SHADER
        .split("fn zr_environment_diffuse_color_normalized(")
        .nth(1)
        .and_then(|source| source.split("fn zr_environment_diffuse_color(").next())
        .expect("environment shader should retain the normalized diffuse IBL owner");

    assert!(
        normalized_diffuse.contains("zr_environment_sh9_color_normalized(normal)"),
        "normalized diffuse IBL must pass its normalized normal through to SH9"
    );
    assert!(
        normalized_diffuse.contains("zr_environment_irradiance_cube_color_normalized(normal)"),
        "normalized diffuse IBL must pass its normalized normal through to the irradiance cube"
    );
    assert!(
        !normalized_diffuse.contains("zr_environment_normalize_or_zero("),
        "the PBR normalized diffuse path must not normalize the normal a second time"
    );
}

#[test]
fn runtime_environment_sh9_diffuse_tracks_runtime_sky_rotation() {
    let normalized_diffuse = ENVIRONMENT_SHADER
        .split("fn zr_environment_diffuse_color_normalized(")
        .nth(1)
        .and_then(|source| source.split("fn zr_environment_diffuse_color(").next())
        .expect("environment shader should retain the normalized diffuse IBL owner");

    assert_eq!(
        normalized_diffuse
            .matches("zr_environment_sh9_color_normalized(normal)")
            .count(),
        2,
        "source and realtime SH9 diffuse must share the rotation-aware normalized owner"
    );

    let rotated_sh9 = ENVIRONMENT_SHADER
        .split("fn zr_environment_sh9_color_normalized(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn zr_environment_irradiance_cube_color_normalized(")
                .next()
        })
        .expect("environment shader should retain the rotation-aware normalized SH9 owner");
    let rotation = rotated_sh9
        .find("let rotated = zr_environment_rotated_direction(normal);")
        .expect("SH9 diffuse must rotate the normalized world normal with the runtime sky");
    let evaluation = rotated_sh9
        .find("zr_environment_sh9_eval_normalized(rotated)")
        .expect("SH9 diffuse must evaluate coefficients in the rotated environment direction");

    assert!(
        rotation < evaluation,
        "runtime sky rotation must be applied before SH9 coefficient evaluation"
    );
    assert!(
        !rotated_sh9.contains("zr_environment_normalize_or_zero("),
        "the rotation-aware SH9 path must reuse the PBR-normalized normal"
    );
}
