use crate::core::framework::render::{ShaderFeatureBits, ShaderPassType};

use super::environment::{wgsl_function_source, wgsl_without_comments};
use super::{
    MaterialShaderTemplateRequest, assemble_material_shader_template, material_template_request,
    standard_material_surface_source_for_features, static_mesh_descriptor,
    validate_material_shader_template_wgsl,
};

#[test]
fn environment_only_pbr_reuses_caller_normalized_surface_inputs() {
    let components = wgsl_function_source(
        include_str!("../../wgsl/zr_environment_only_pbr.wgsl"),
        "fn zr_environment_pbr_components(",
    );
    let forward_shading = wgsl_function_source(
        include_str!("../../wgsl/zr_shading_environment_only_pbr.wgsl"),
        "fn shade_forward(",
    );
    let deferred_shading = wgsl_function_source(
        include_str!(
            "../../../scene/scene_renderer/deferred/shaders/deferred_environment_only_pbr.wgsl"
        ),
        "fn shade_deferred_environment_only_pbr(",
    );

    assert!(
        components.contains("view_dir_normalized: vec3<f32>"),
        "environment-only PBR components must declare their normalized-view contract"
    );
    assert!(
        components.contains("normal_normalized: vec3<f32>"),
        "environment-only PBR components must declare their normalized-normal contract"
    );
    assert!(
        components.contains("let normal = normal_normalized;"),
        "environment-only PBR components must reuse the caller-normalized surface normal"
    );
    assert!(
        !components.contains("zr_environment_normalize_or_zero(normal_normalized)"),
        "environment-only PBR components must not normalize the surface normal twice"
    );
    assert!(
        components.contains("let view_dir = view_dir_normalized;"),
        "environment-only PBR components must reuse the caller-normalized view direction"
    );
    assert!(
        !components.contains("zr_environment_normalize_or_zero(view_dir_normalized)"),
        "environment-only PBR components must not normalize the view direction twice"
    );
    assert!(
        components.contains("all(view_dir == vec3<f32>(0.0))"),
        "environment-only PBR components must retain the zero-view rejection"
    );
    assert!(
        components.contains("all(normal == vec3<f32>(0.0))"),
        "environment-only PBR components must retain the zero-normal rejection"
    );
    assert!(
        components.contains("dielectric_f0: vec3<f32>"),
        "environment-only PBR components must preserve the material dielectric F0"
    );
    assert!(
        components.contains("base_color,\n        dielectric_f0,\n        has_global_environment,"),
        "environment-only PBR components must forward dielectric F0 to the shared split-sum BRDF"
    );
    assert!(
        forward_shading.contains("let view_dir_ws = zr_pbr_view_direction_ws(ctx.position_ws);")
    );
    assert!(deferred_shading.contains("let view_dir = zr_pbr_view_direction_ws(world_position);"));
    let forward_ibl_call = forward_shading
        .find("let environment_lights = zr_environment_pbr_indirect(")
        .expect("environment-only Forward must retain its IBL call");
    let deferred_ibl_call = deferred_shading
        .find("let environment_lights = zr_environment_pbr_indirect_with_dielectric_f0_normalized(")
        .expect("environment-only deferred must retain its IBL call");
    let forward_ibl_call_end = forward_shading[forward_ibl_call..]
        .find("\n    );")
        .map(|offset| forward_ibl_call + offset)
        .expect("environment-only Forward IBL call must close");
    let deferred_ibl_call_end = deferred_shading[deferred_ibl_call..]
        .find("\n    );")
        .map(|offset| deferred_ibl_call + offset)
        .expect("environment-only deferred IBL call must close");
    let forward_normal = forward_shading
        .find("let world_normal = zr_normalize_or_zero(surface.normal_ws);")
        .expect("environment-only Forward must normalize its interpolated surface normal once");
    assert!(
        forward_normal < forward_ibl_call,
        "environment-only Forward must prepare its normal before issuing the IBL query"
    );
    assert!(
        forward_shading[forward_ibl_call..forward_ibl_call_end].contains(
            "ctx.position_ws,\n        world_normal,\n        view_dir_ws,\n        surface.roughness,",
        ),
        "environment-only Forward must pass its normalized surface normal and view direction to IBL"
    );
    assert!(
        forward_shading[forward_ibl_call..forward_ibl_call_end]
            .contains("diffuse_color,\n        surface.dielectric_f0,\n        surface.occlusion,"),
        "environment-only Forward must preserve the authored dielectric F0"
    );
    assert!(
        deferred_shading[deferred_ibl_call..deferred_ibl_call_end]
            .contains("world_position,\n        normal,\n        view_dir,\n        roughness,",),
        "environment-only deferred must pass normalized GBuffer normal and view direction to IBL"
    );
    assert!(
        deferred_shading[deferred_ibl_call..deferred_ibl_call_end]
            .contains("diffuse_color,\n        vec3<f32>(0.04),\n        occlusion,"),
        "environment-only deferred must make its GBuffer-limited dielectric F0 fallback explicit"
    );

    assert!(forward_shading.contains("let diffuse_energy = vec3<f32>("));
    assert!(
        forward_shading.contains("zr_surface_metallic_diffuse_energy_scale(surface.metallic),")
    );
    assert!(deferred_shading.contains("let diffuse_energy = vec3<f32>("));
    assert!(
        deferred_shading.contains("zr_surface_metallic_diffuse_energy_scale(metallic)"),
        "environment-only Deferred must apply the shared metallic diffuse-energy owner"
    );
    assert!(!forward_shading.contains("zr_pbr_diffuse_energy_scale("));
    assert!(!deferred_shading.contains("zr_pbr_diffuse_energy_scale("));
    assert!(
        forward_shading.contains("let diffuse_color = zr_pbr_base_color(surface.base_color.rgb);"),
        "environment-only Forward must consume physical Standard-PBR diffuse reflectance"
    );
    assert!(
        deferred_shading.contains("let diffuse_color = zr_pbr_base_color(albedo.rgb);"),
        "environment-only deferred must consume physical Standard-PBR diffuse reflectance"
    );
}

#[test]
fn pbr_view_direction_has_one_owner_and_skips_redundant_endpoint_work() {
    let common = include_str!("../../includes/zr_pbr_common.wgsl");
    let view_direction = wgsl_function_source(common, "fn zr_pbr_view_direction_ws(");
    let weight = view_direction
        .find("let camera_direction_weight = clamp(scene.camera_view_direction.w, 0.0, 1.0);")
        .expect("the common owner must clamp its camera-direction blend once");
    let perspective_endpoint = view_direction
        .find("if (camera_direction_weight <= 0.0) {")
        .expect("the common owner must return early for perspective cameras");
    let orthographic_endpoint = view_direction
        .find("if (camera_direction_weight >= 1.0) {")
        .expect("the common owner must return early for orthographic cameras");
    let perspective = view_direction
        .find("let perspective_view_dir = zr_pbr_common_normalize_or_zero(")
        .expect("the common owner must retain the mixed-camera perspective path");
    let mixed = view_direction
        .find("return zr_pbr_common_normalize_or_zero(mix(")
        .expect("the common owner must normalize the mixed-camera path");
    assert!(
        weight < perspective_endpoint
            && perspective_endpoint < orthographic_endpoint
            && orthographic_endpoint < perspective
            && perspective < mixed,
        "the common owner must avoid mixed-camera work at both endpoints"
    );

    let view_direction_consumers = [
        (
            "environment-only Forward",
            include_str!("../../wgsl/zr_shading_environment_only_pbr.wgsl"),
            "fn zr_scene_view_dir_ws(",
            "let view_dir_ws = zr_pbr_view_direction_ws(ctx.position_ws);",
        ),
        (
            "advanced Standard-PBR Forward",
            include_str!("../../wgsl/zr_shading_standard_pbr.wgsl"),
            "fn zr_scene_view_dir_ws(",
            "let view_dir_ws = zr_pbr_view_direction_ws(ctx.position_ws);",
        ),
        (
            "basic Standard-PBR Forward",
            include_str!("../../wgsl/zr_shading_standard_pbr_basic.wgsl"),
            "fn zr_scene_view_dir_ws(",
            "let view_dir_ws = zr_pbr_view_direction_ws(ctx.position_ws);",
        ),
        (
            "environment-only deferred",
            include_str!(
                "../../../scene/scene_renderer/deferred/shaders/deferred_environment_only_pbr.wgsl"
            ),
            "fn scene_view_dir_ws(",
            "let view_dir = zr_pbr_view_direction_ws(world_position);",
        ),
        (
            "generic deferred",
            include_str!("../../../scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl"),
            "fn scene_view_dir_ws(",
            "let view_dir = zr_pbr_view_direction_ws(world_position);",
        ),
        (
            "fallback mesh",
            include_str!("../../../scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl"),
            "fn scene_view_dir_ws(",
            "let view_dir = zr_pbr_view_direction_ws(input.world_position);",
        ),
    ];

    for (label, source, legacy_owner, expected_call) in view_direction_consumers {
        assert!(
            !source.contains(legacy_owner),
            "{label} must not retain a second view-direction owner"
        );
        assert!(
            source.contains(expected_call),
            "{label} must reuse the common view-direction owner"
        );
    }
}

#[test]
fn normalizer_contract_has_one_finite_length_owner() {
    let common_module = include_str!("../../includes/zr_pbr_common.wgsl");
    let common = wgsl_function_source(common_module, "fn zr_pbr_common_normalize_or_zero(");
    assert!(
        common_module.contains("const ZR_PBR_COMMON_MAX_NORMALIZABLE_LENGTH: f32 = 3.4e38;"),
        "the shared normalizer must bound non-finite lengths without backend-specific builtins"
    );
    assert!(
        common.contains("let is_finite_length = value_length > 0.000001")
            && common.contains("&& value_length < ZR_PBR_COMMON_MAX_NORMALIZABLE_LENGTH;"),
        "the shared normalizer must reject zero, NaN, and infinite lengths"
    );
    assert!(
        common.contains("is_finite_length),"),
        "the shared normalizer must select the zero fallback for invalid lengths"
    );

    for (label, source, signature) in [
        (
            "environment",
            include_str!("../../wgsl/zr_environment_core.wgsl"),
            "fn zr_environment_normalize_or_zero(",
        ),
        (
            "PBR extras",
            include_str!("../../includes/zr_pbr_extras_core.wgsl"),
            "fn zr_pbr_normalize_or_zero(",
        ),
        (
            "surface",
            include_str!("../../wgsl/zr_surface_types.wgsl"),
            "fn zr_normalize_or_zero(",
        ),
        (
            "environment-only Deferred",
            include_str!(
                "../../../scene/scene_renderer/deferred/shaders/deferred_environment_only_pbr.wgsl"
            ),
            "fn normalize_or_zero(",
        ),
        (
            "generic Deferred",
            include_str!("../../../scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl"),
            "fn normalize_or_zero(",
        ),
        (
            "fallback mesh",
            include_str!("../../../scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl"),
            "fn normalize_or_zero(",
        ),
    ] {
        let normalizer = wgsl_function_source(source, signature);
        assert!(
            normalizer.contains("return zr_pbr_common_normalize_or_zero(value);"),
            "{label} normalizer must delegate finite-length handling to the shared owner"
        );
        assert!(
            !normalizer.contains("let value_length = length(value);"),
            "{label} normalizer must not duplicate length validation"
        );
    }

    for (label, source) in [
        ("Forward", include_str!("../assemble.rs")),
        ("GBuffer", include_str!("../deferred_gbuffer.rs")),
        ("TAA reactive mask", include_str!("../taa_reactive_mask.rs")),
    ] {
        let common = source
            .find("pbr_common_include())")
            .unwrap_or_else(|| panic!("{label} assembly must include the shared normalizer owner"));
        let surface = source
            .find("surface_types_include())")
            .unwrap_or_else(|| panic!("{label} assembly must include the surface contract"));
        assert!(
            common < surface,
            "{label} assembly must emit the shared normalizer before surface consumers"
        );
    }
}

#[test]
fn fallback_mesh_deformed_frame_uses_the_finite_normalizer_owner() {
    let fallback = include_str!("../../../scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl");
    let skinned_normal = wgsl_function_source(fallback, "fn skin_vertex_normal(");
    let skinned_tangent = wgsl_function_source(fallback, "fn skin_vertex_tangent(");
    let sampled_normal = wgsl_function_source(fallback, "fn sampled_world_normal(");

    assert!(skinned_normal.contains("return zr_pbr_common_normalize_or_zero(skinned);"));
    assert!(skinned_tangent.contains("return zr_pbr_common_normalize_or_zero(skinned);"));
    assert!(sampled_normal.contains("return normalize_or_zero(world_normal);"));
    for legacy in [
        "return skinned / normal_length;",
        "return skinned / tangent_length;",
        "return normalize(world_normal);",
    ] {
        assert!(
            !fallback.contains(legacy),
            "fallback mesh must not retain non-finite normalization path `{legacy}`"
        );
    }
}

#[test]
fn generic_pbr_indirect_reuses_caller_normalized_surface_inputs() {
    let environment = include_str!("../../wgsl/zr_environment.wgsl");
    let normalized_components =
        wgsl_function_source(environment, "fn zr_environment_pbr_components_normalized(");
    let normalized_indirect =
        wgsl_function_source(environment, "fn zr_environment_pbr_indirect_normalized(");

    assert!(
        normalized_components.contains("normal_normalized: vec3<f32>"),
        "the normalized generic IBL entry must declare its normal contract"
    );
    assert!(
        normalized_components.contains("view_dir_normalized: vec3<f32>"),
        "the normalized generic IBL entry must declare its view contract"
    );
    assert!(
        !normalized_components.contains("zr_environment_normalize_or_zero(normal_normalized)"),
        "the normalized generic IBL entry must not normalize its normal a second time"
    );
    assert!(
        !normalized_components.contains("zr_environment_normalize_or_zero(view_dir_normalized)"),
        "the normalized generic IBL entry must not normalize its view a second time"
    );
    assert!(
        normalized_indirect.contains("zr_environment_pbr_components_normalized("),
        "the normalized indirect wrapper must preserve the shared PBR components path"
    );

    let consumers = [
        (
            "advanced Standard-PBR Forward",
            include_str!("../../wgsl/zr_shading_standard_pbr.wgsl"),
            "fn shade_forward(",
            "ctx.position_ws,\n        world_normal,\n        view_dir_ws,",
        ),
        (
            "basic Standard-PBR Forward",
            include_str!("../../wgsl/zr_shading_standard_pbr_basic.wgsl"),
            "fn shade_forward(",
            "ctx.position_ws,\n        world_normal,\n        view_dir_ws,",
        ),
        (
            "generic deferred lighting",
            include_str!("../../../scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl"),
            "fn shade_deferred_lit(",
            "world_position,\n        normal,\n        view_dir,",
        ),
        (
            "fallback mesh",
            include_str!("../../../scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl"),
            "fn fs_main(",
            "input.world_position,\n        world_normal,\n        view_dir,",
        ),
    ];

    for (label, source, signature, normalized_inputs) in consumers {
        let consumer = wgsl_function_source(source, signature);
        let indirect = consumer
            .find("zr_environment_pbr_indirect_normalized(")
            .unwrap_or_else(|| panic!("{label} must call the normalized generic IBL entry"));
        let call_end = consumer[indirect..]
            .find("\n    );")
            .map(|offset| indirect + offset)
            .unwrap_or_else(|| panic!("{label} normalized IBL call must close"));
        assert!(
            consumer[indirect..call_end].contains(normalized_inputs),
            "{label} must reuse its normalized normal and view direction for IBL"
        );
        assert!(
            !consumer.contains("zr_environment_pbr_indirect(\n"),
            "{label} must not retain the defensive IBL entry after normalizing its own inputs"
        );
    }

    let deferred_subsurface = wgsl_function_source(
        include_str!("../../../scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl"),
        "fn shade_deferred_subsurface_components(",
    );
    assert!(
        deferred_subsurface.contains(
            "zr_environment_pbr_components_normalized(\n        world_position,\n        normal,\n        view_dir,"
        ),
        "deferred subsurface must reuse its decoded normal and view direction for environment components"
    );
    assert!(
        !deferred_subsurface.contains("zr_environment_pbr_components(\n"),
        "deferred subsurface must not re-normalize its environment inputs"
    );
}

#[test]
fn pbr_punctual_lights_reuse_the_existing_light_vector_for_cone_visibility() {
    let sources = [
        (
            "advanced Standard-PBR Forward",
            include_str!("../../wgsl/zr_shading_standard_pbr.wgsl"),
            "fn zr_standard_pbr_punctual_light_visibility(",
            "zr_normalize_or_zero",
        ),
        (
            "basic Standard-PBR Forward",
            include_str!("../../wgsl/zr_shading_standard_pbr_basic.wgsl"),
            "fn zr_standard_pbr_punctual_light_visibility(",
            "zr_normalize_or_zero",
        ),
        (
            "generic deferred",
            include_str!("../../../scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl"),
            "fn punctual_light_visibility(",
            "normalize_or_zero",
        ),
        (
            "fallback mesh",
            include_str!("../../../scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl"),
            "fn punctual_light_visibility(",
            "normalize_or_zero",
        ),
    ];

    for (label, source, signature, normalize) in sources {
        let visibility = wgsl_function_source(source, signature);
        assert!(
            visibility.contains("light_vector_to_light: vec3<f32>"),
            "{label} must receive the already-normalized surface-to-light vector"
        );
        assert!(
            visibility.contains("range: f32"),
            "{label} must consume the caller's already-checked light range"
        );
        assert!(
            visibility.contains("-light_vector_to_light,"),
            "{label} must reverse the existing light vector for cone visibility"
        );
        assert!(
            visibility.contains("distance_to_light >"),
            "{label} must preserve the zero-safe cone direction for near-zero distances"
        );
        assert!(
            !visibility.contains("world_position - light.position_range.xyz"),
            "{label} must not rebuild the light-to-surface vector"
        );
        assert!(
            !visibility.contains(&format!("{normalize}(world_position")),
            "{label} must not normalize a second point-light direction"
        );
    }
}

#[test]
fn pbr_punctual_lights_reject_out_of_range_before_normalizing_the_light_vector() {
    let callers = [
        (
            "advanced Standard-PBR Forward",
            include_str!("../../wgsl/zr_shading_standard_pbr.wgsl"),
            "fn zr_standard_pbr_shade_gpu_light_index(",
        ),
        (
            "basic Standard-PBR Forward",
            include_str!("../../wgsl/zr_shading_standard_pbr_basic.wgsl"),
            "fn zr_standard_pbr_shade_gpu_light_index(",
        ),
        (
            "generic deferred lighting",
            include_str!("../../../scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl"),
            "fn shade_gpu_light_index(",
        ),
        (
            "generic deferred components",
            include_str!("../../../scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl"),
            "fn shade_gpu_light_index_components(",
        ),
        (
            "fallback mesh",
            include_str!("../../../scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl"),
            "fn shade_gpu_light_index(",
        ),
    ];

    for (label, source, signature) in callers {
        let caller = wgsl_function_source(source, signature);
        let distance = caller
            .find("let distance_to_light = length(to_light);")
            .unwrap_or_else(|| panic!("{label} must compute point-light distance once"));
        let range = caller
            .find("let range = max(light.position_range.w,")
            .unwrap_or_else(|| panic!("{label} must resolve the light range before normalization"));
        let out_of_range = caller
            .find("if (distance_to_light >= range) {")
            .unwrap_or_else(|| panic!("{label} must retain out-of-range rejection"));
        let light_vector = caller
            .find("let light_vector = to_light / max(distance_to_light,")
            .unwrap_or_else(|| panic!("{label} must retain one surface-to-light normalization"));
        let visibility = caller
            .find("let visibility =")
            .unwrap_or_else(|| panic!("{label} must resolve punctual visibility"));
        let visibility_call = caller[visibility..]
            .find("punctual_light_visibility(")
            .map(|offset| visibility + offset)
            .unwrap_or_else(|| {
                panic!("{label} must pass the existing light vector to punctual visibility")
            });
        let visibility_arguments = &caller[visibility_call..];
        let passed_light_vector = visibility_arguments
            .find("light_vector")
            .unwrap_or_else(|| panic!("{label} must pass the existing light vector"));
        let passed_range = visibility_arguments
            .find("range")
            .unwrap_or_else(|| panic!("{label} must pass the checked light range"));

        assert!(
            distance < range
                && range < out_of_range
                && out_of_range < light_vector
                && light_vector < visibility
                && visibility <= visibility_call
                && passed_light_vector < passed_range,
            "{label} must reject out-of-range lights before constructing the normalized vector"
        );
    }
}

#[test]
fn environment_only_forward_specialization_excludes_unreachable_environment_api() {
    let generic = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("generic Forward template assembly");
    let features = ShaderFeatureBits::new(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR);
    let surface = standard_material_surface_source_for_features(features, 0.5);
    let specialized = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            static_mesh_descriptor(),
            ShaderPassType::Forward,
            surface.source,
            surface.entry_point,
        )
        .with_features(surface.features),
    )
    .expect("environment-only Standard-PBR Forward template assembly");
    let generic_source = wgsl_without_comments(&generic.wgsl_source);
    let specialized_source = wgsl_without_comments(&specialized.wgsl_source);

    for required in [
        "zr_environment_sky_reflection_color(",
        "zr_environment_diffuse_color_normalized(",
        "zr_environment_env_brdf_lut(",
        "zr_environment_perfect_specular_direction_normalized(",
        "zr_environment_dominant_specular_direction_normalized(",
        "if (zr_environment_is_source_cubemap() || zr_environment_is_realtime_ibl()) {",
    ] {
        assert!(
            specialized_source.contains(required),
            "environment-only Forward must retain global IBL `{required}`"
        );
    }
    for excluded_source in [
        "@group(1) @binding(16)",
        "@group(1) @binding(17)",
        "@group(1) @binding(18)",
        "@group(1) @binding(29)",
        "@group(1) @binding(30)",
        "fn zr_environment_select_probes(",
        "fn zr_environment_probe_color(",
        "fn zr_environment_planar_reflection(",
        "fn zr_environment_reflection_color_after_planar(",
        "fn zr_environment_reflection_color(",
        "fn zr_environment_reflection_color_normalized(",
        "fn zr_environment_fix_source_cube_lookup(",
        "fn zr_environment_source_cube_color_at_lod(",
        "fn zr_environment_specular_pmrem_color_at_lod(",
        "fn zr_environment_env_brdf_approx(",
        "fn zr_environment_sh9_eval(",
        "fn zr_environment_irradiance_cube_color(",
        "fn zr_environment_procedural_sky_color(",
        "fn zr_environment_sky_color(",
        "fn zr_environment_diffuse_color(",
    ] {
        assert!(
            !specialized_source.contains(excluded_source),
            "environment-only Forward must exclude unreachable source `{excluded_source}`"
        );
        assert!(
            generic_source.contains(excluded_source),
            "generic Forward must retain `{excluded_source}`"
        );
    }
    let generic_environment_hash = generic
        .include_tokens
        .iter()
        .zip(&generic.include_content_hashes)
        .find_map(|(token, hash)| (token == "zr_environment.wgsl").then_some(hash))
        .expect("generic Forward should retain the canonical environment include token");
    let specialized_environment_hash = specialized
        .include_tokens
        .iter()
        .zip(&specialized.include_content_hashes)
        .find_map(|(token, hash)| (token == "zr_environment.wgsl").then_some(hash))
        .expect("environment-only Forward should retain the canonical environment include token");
    assert_ne!(
        generic_environment_hash, specialized_environment_hash,
        "generic and environment-only composites must have distinct environment content hashes"
    );
    assert!(
        specialized.wgsl_source.len() * 2 <= generic.wgsl_source.len(),
        "global-only IBL specialization should remove at least half of comparable WGSL, generic={} specialized={}",
        generic.wgsl_source.len(),
        specialized.wgsl_source.len(),
    );
    validate_material_shader_template_wgsl(&specialized.wgsl_source)
        .expect("environment-only Standard-PBR Forward WGSL should validate");
}
