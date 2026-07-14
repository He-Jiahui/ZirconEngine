use super::*;

#[test]
fn render_shader_template_assembles_standard_material_surface_source() {
    let static_mesh = static_mesh_descriptor();
    let material = standard_material_descriptor();
    let surface_source = standard_material_surface_source(&material);

    assert!(surface_source
        .features
        .contains(ShaderFeatureBits::ALPHA_TEST));
    assert!(surface_source
        .features
        .contains(ShaderFeatureBits::RECEIVE_SHADOWS));
    assert!(surface_source
        .features
        .contains(ShaderFeatureBits::DOUBLE_SIDED));
    assert!(!surface_source
        .features
        .contains(ShaderFeatureBits::HAS_NORMAL_TEXTURE));
    assert!(!surface_source
        .features
        .contains(ShaderFeatureBits::PBR_CLEARCOAT));
    assert!(!surface_source
        .features
        .contains(ShaderFeatureBits::PBR_ANISOTROPY));
    assert!(!surface_source
        .features
        .contains(ShaderFeatureBits::PBR_TRANSMISSION));

    let assembly = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            static_mesh,
            ShaderPassType::Forward,
            surface_source.source.clone(),
            surface_source.entry_point,
        )
        .with_features(surface_source.features),
    )
    .expect("standard material template assembly");

    assert!(assembly.wgsl_source.contains("fn zr_material_surface("));
    assert!(!assembly
        .wgsl_source
        .contains("fn standard_material_surface("));
    assert!(assembly
        .wgsl_source
        .contains("@group(2) @binding(0) var<uniform> standard_material_properties"));
    assert!(assembly
        .wgsl_source
        .contains("standard_material_metallic_roughness_tex"));
    assert!(assembly
        .wgsl_source
        .contains("ZR_STANDARD_MATERIAL_SURFACE_MIN_ROUGHNESS"));
    assert!(assembly
        .wgsl_source
        .contains("standard_material_normal_tex"));
    assert!(assembly
        .wgsl_source
        .contains("fn standard_material_sampled_normal"));
    assert!(assembly
        .wgsl_source
        .contains("if (!ZR_FEATURE_HAS_NORMAL_TEXTURE)"));
    assert!(assembly
        .wgsl_source
        .contains("const ZR_STANDARD_MATERIAL_ALPHA_CUTOFF: f32 = 0.50000000;"));
    assert!(assembly
        .wgsl_source
        .contains("fn standard_material_alpha_cutoff() -> f32"));
    assert!(assembly
        .wgsl_source
        .contains("let uniform_cutoff = clamp(standard_material_properties.data8.z, 0.0, 1.0);"));
    assert!(assembly
        .wgsl_source
        .contains("surface.alpha_cutoff = standard_material_alpha_cutoff();"));
    assert!(assembly
        .wgsl_source
        .contains("surface.unlit = standard_material_properties.data0.w;"));
    assert!(assembly
        .wgsl_source
        .contains("surface.shading_model_id = standard_material_shading_model_id();"));
    assert!(assembly
        .wgsl_source
        .contains("zr_apply_alpha_clip(surface);"));
    assert!(assembly
        .wgsl_source
        .contains("// include: zr_scene_runtime.wgsl"));
    assert!(assembly
        .wgsl_source
        .contains("// include: zr_gpu_scene.wgsl"));
    assert_include_token!(assembly, "zr_environment.wgsl");
    assert_include_token!(assembly, "zr_light_grid.wgsl");
    assert_include_token!(assembly, "zr_shadow.wgsl");
    assert!(assembly
        .wgsl_source
        .contains("sky_horizon_color: vec4<f32>"));
    assert!(assembly
        .wgsl_source
        .contains("environment_params: vec4<f32>"));
    assert!(assembly
        .wgsl_source
        .contains("environment_sample_params: vec4<f32>"));
    assert!(assembly
        .wgsl_source
        .contains("camera_world_position: vec4<f32>"));
    assert!(assembly
        .wgsl_source
        .contains("camera_view_direction: vec4<f32>"));
    let scene_uniform = assembly
        .wgsl_source
        .split("struct SceneUniform {")
        .nth(1)
        .and_then(|tail| tail.split("};").next())
        .expect("standard material shader should declare SceneUniform");
    assert!(!scene_uniform.contains("environment_sh9"));
    assert!(assembly
        .wgsl_source
        .contains("override ZR_ENV_DIFFUSE_IEM: bool = false;"));
    assert!(assembly
        .wgsl_source
        .contains("@group(0) @binding(1) var zr_environment_source_cube: texture_cube<f32>;"));
    assert!(assembly
        .wgsl_source
        .contains("@group(0) @binding(2) var zr_environment_sampler: sampler;"));
    assert!(assembly
        .wgsl_source
        .contains("@group(0) @binding(3) var zr_environment_brdf_lut: texture_2d<f32>;"));
    assert!(assembly.wgsl_source.contains(
        "@group(0) @binding(4) var zr_environment_specular_pmrem_cube: texture_cube<f32>;"
    ));
    assert!(assembly
        .wgsl_source
        .contains("@group(0) @binding(5) var zr_environment_irradiance_cube: texture_cube<f32>;"));
    assert!(assembly
        .wgsl_source
        .contains("@group(0) @binding(6) var<uniform> zr_environment_sh9: ZrEnvironmentSh9;"));
    assert!(assembly.wgsl_source.contains("textureSampleLevel("));
    assert!(assembly
        .wgsl_source
        .contains("zr_environment_specular_pmrem_cube"));
    assert!(assembly
        .wgsl_source
        .contains("zr_environment_irradiance_cube"));
    assert!(assembly
        .wgsl_source
        .contains("fn zr_environment_mip_from_roughness"));
    assert!(assembly.wgsl_source.contains("fn zr_environment_sh9_eval"));
    assert!(assembly
        .wgsl_source
        .contains("fn zr_environment_irradiance_cube_color"));
    assert!(assembly
        .wgsl_source
        .contains("fn zr_environment_env_brdf_lut"));
    assert!(assembly
        .wgsl_source
        .contains("fn zr_environment_env_brdf_approx"));
    assert!(assembly
        .wgsl_source
        .contains("fn zr_environment_pbr_indirect"));
    assert!(assembly
        .wgsl_source
        .contains("@group(1) @binding(20) var<uniform> zr_light_grid_params"));
    assert!(assembly
        .wgsl_source
        .contains("@group(1) @binding(8) var zr_shadow_atlas: texture_depth_2d"));
    assert!(assembly.wgsl_source.contains("fn zr_build_vertex_output("));
    assert!(assembly
        .wgsl_source
        .contains("let position_ws = world_from_local * vec4<f32>(position_os, 1.0);"));
    assert!(assembly
        .wgsl_source
        .contains("output.position_ws = position_ws.xyz;"));
    assert!(assembly.wgsl_source.contains(
        "output.normal_ws = zr_normalize_or_zero((world_from_local * vec4<f32>(normal_os, 0.0)).xyz);"
    ));
    assert!(assembly.wgsl_source.contains(
        "output.tangent_ws = zr_normalize_or_zero((world_from_local * vec4<f32>(tangent_os.xyz, 0.0)).xyz);"
    ));
    assert!(assembly
        .wgsl_source
        .contains("output.tangent_handedness = select(-1.0, 1.0, tangent_os.w >= 0.0);"));
    assert!(assembly
        .wgsl_source
        .contains("output.tint = zr_gpu_scene_tint(instance_index);"));
    assert!(assembly
        .wgsl_source
        .contains("output.shadow_params = zr_gpu_scene_shadow_params(instance_index);"));
    assert!(assembly.wgsl_source.contains("input.tangent_handedness"));
    assert!(assembly.wgsl_source.contains("input.tint * input.color"));
    assert!(assembly.wgsl_source.contains("struct ZrShadingContext"));
    assert!(assembly
        .wgsl_source
        .contains("fn zr_build_shading_context(input: ZrVertexOutput) -> ZrShadingContext"));
    assert!(assembly
        .wgsl_source
        .contains("shade_forward(surface, zr_build_shading_context(input))"));
    assert!(assembly.wgsl_source.contains(
        "fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> vec3<f32>"
    ));
    assert!(assembly
        .wgsl_source
        .contains("ZR_FEATURE_RECEIVE_SHADOWS && ctx.shadow_params.z > 0.5"));
    assert!(assembly
        .wgsl_source
        .contains("zr_gpu_light_shadow_visibility(light, light_type, ctx.position_ws, view_z)"));
    assert!(assembly
        .wgsl_source
        .contains("const ZR_SHADING_MODEL_UNLIT_ID: u32 = 0u;"));
    assert!(assembly
        .wgsl_source
        .contains("const ZR_SHADING_MODEL_BLINN_PHONG_ID: u32 = 1u;"));
    assert!(assembly
        .wgsl_source
        .contains("const ZR_SHADING_MODEL_STANDARD_PBR_ID: u32 = 2u;"));
    assert!(assembly
        .wgsl_source
        .contains("surface.shading_model_id == ZR_SHADING_MODEL_UNLIT_ID"));
    assert!(assembly
        .wgsl_source
        .contains("surface.shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID"));
    assert!(assembly
        .wgsl_source
        .contains("fn zr_standard_pbr_shade_blinn_phong_light_vector"));
    assert!(assembly
        .wgsl_source
        .contains("let environment_lights = zr_environment_pbr_indirect("));
    assert!(assembly
        .wgsl_source
        .contains("fn zr_scene_view_dir_ws(position_ws: vec3<f32>) -> vec3<f32>"));
    assert!(assembly
        .wgsl_source
        .contains("let view_dir_ws = zr_scene_view_dir_ws(ctx.position_ws);"));
    assert!(!assembly
        .wgsl_source
        .contains("surface.normal_ws,\n        vec3<f32>(0.0, 0.0, 1.0),"));
    assert!(assembly
        .wgsl_source
        .contains("surface.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID"));
    assert!(assembly.wgsl_source.contains("@location(2) uv0: vec2<f32>"));
    assert!(assembly
        .wgsl_source
        .contains("@location(3) joints: vec4<u32>"));
    assert!(assembly
        .wgsl_source
        .contains("@location(4) weights: vec4<f32>"));
    assert!(assembly
        .wgsl_source
        .contains("@location(5) tangent: vec4<f32>"));
    assert!(assembly
        .wgsl_source
        .contains("@location(6) color: vec4<f32>"));
    assert!(assembly.wgsl_source.contains("@location(7) uv1: vec2<f32>"));
    assert!(assembly.wgsl_source.contains("@location(3) uv1: vec2<f32>"));
    assert!(assembly
        .wgsl_source
        .contains("@location(4) tangent_ws: vec3<f32>"));
    assert!(assembly.wgsl_source.contains("input.uv1"));
    assert!(assembly
        .wgsl_source
        .contains("fetch_tangent(v, instance_index)"));
    assert!(assembly
        .wgsl_source
        .contains("const ZR_FEATURE_ALPHA_TEST: bool = true;"));
    assert!(assembly
        .wgsl_source
        .contains("const ZR_FEATURE_RECEIVE_SHADOWS: bool = true;"));
    assert!(assembly
        .wgsl_source
        .contains("const ZR_FEATURE_DOUBLE_SIDED: bool = true;"));
    assert!(assembly
        .wgsl_source
        .contains("const ZR_FEATURE_HAS_NORMAL_TEXTURE: bool = false;"));
    assert!(assembly
        .wgsl_source
        .contains("const ZR_FEATURE_PBR_CLEARCOAT: bool = false;"));
    assert!(assembly
        .wgsl_source
        .contains("const ZR_FEATURE_PBR_ANISOTROPY: bool = false;"));
    assert!(assembly
        .wgsl_source
        .contains("const ZR_FEATURE_PBR_TRANSMISSION: bool = false;"));
}

#[test]
fn render_shader_template_projects_advanced_pbr_features() {
    let mut material = standard_material_descriptor();
    material.advanced_features = StandardPbrMaterialFeatures {
        clearcoat: 0.8,
        anisotropy_strength: 0.6,
        specular_transmission: 0.7,
        ..Default::default()
    };

    let surface_source = standard_material_surface_source(&material);

    assert!(surface_source
        .features
        .contains(ShaderFeatureBits::PBR_CLEARCOAT));
    assert!(surface_source
        .features
        .contains(ShaderFeatureBits::PBR_ANISOTROPY));
    assert!(surface_source
        .features
        .contains(ShaderFeatureBits::PBR_TRANSMISSION));

    let assembly = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            static_mesh_descriptor(),
            ShaderPassType::Forward,
            surface_source.source,
            surface_source.entry_point,
        )
        .with_features(surface_source.features),
    )
    .expect("advanced Standard PBR template assembly");

    assert!(assembly
        .wgsl_source
        .contains("const ZR_FEATURE_PBR_CLEARCOAT: bool = true;"));
    assert!(assembly
        .wgsl_source
        .contains("const ZR_FEATURE_PBR_ANISOTROPY: bool = true;"));
    assert!(assembly
        .wgsl_source
        .contains("const ZR_FEATURE_PBR_TRANSMISSION: bool = true;"));
    for projection in [
        "surface.clearcoat = select(0.0, clamp(standard_material_properties.data9.x",
        "surface.anisotropy_rotation = select(0.0, standard_material_properties.data9.w",
        "surface.specular_transmission = select(0.0, clamp(standard_material_properties.data10.x",
        "surface.ior = select(1.5, max(standard_material_properties.data10.w",
        "surface.attenuation_color = select(vec3<f32>(1.0), clamp(standard_material_properties.data11.rgb",
        "surface.attenuation_distance = select(1.0e30, max(standard_material_properties.data11.w",
    ] {
        assert!(
            assembly.wgsl_source.contains(projection),
            "advanced PBR template is missing projection `{projection}`"
        );
    }
    assert_include_token!(assembly, "zr_pbr_extras.wgsl");
    assert!(assembly
        .wgsl_source
        .contains("@group(2) @binding(11) var standard_material_clearcoat_normal_tex"));
    assert!(assembly
        .wgsl_source
        .contains("@group(2) @binding(12) var standard_material_clearcoat_normal_sampler"));
    assert!(assembly.wgsl_source.contains("fn zr_aniso_ggx"));
    assert!(assembly.wgsl_source.contains("fn zr_clearcoat_lobe"));
    assert!(assembly.wgsl_source.contains("fn zr_transmission_btdf"));
    assert!(assembly
        .wgsl_source
        .contains("@group(1) @binding(31) var zr_transmission_scene_color"));
    assert!(assembly
        .wgsl_source
        .contains("fn zr_pbr_screen_space_transmission"));
    assert!(assembly
        .wgsl_source
        .contains("let transmission_source = select("));
    assert!(assembly.wgsl_source.contains("environment_lighting,"));
    assert!(assembly.wgsl_source.contains("scene_color_sample.rgb,"));
    assert!(assembly.wgsl_source.contains("scene_color_sample.a > 0.0,"));
    validate_material_shader_template_wgsl(&assembly.wgsl_source)
        .expect("advanced Standard PBR WGSL should validate");
}

#[test]
fn render_shader_template_marks_standard_material_normal_texture_feature() {
    let static_mesh = static_mesh_descriptor();
    let mut material = standard_material_descriptor();
    material.normal_texture = Some(AssetReference::from_locator(
        ResourceLocator::parse("res://textures/material-normal.png").expect("normal locator"),
    ));
    let surface_source = standard_material_surface_source(&material);

    assert!(surface_source
        .features
        .contains(ShaderFeatureBits::HAS_NORMAL_TEXTURE));

    let assembly = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            static_mesh,
            ShaderPassType::Forward,
            surface_source.source,
            surface_source.entry_point,
        )
        .with_features(surface_source.features),
    )
    .expect("standard material template assembly with normal map");

    assert!(assembly
        .wgsl_source
        .contains("const ZR_FEATURE_HAS_NORMAL_TEXTURE: bool = true;"));
}
