use super::*;

fn toon_shading_model_descriptor() -> ShadingModelDescriptor {
    ShadingModelDescriptor::new(
        ShadingModelId::new(16),
        "toon",
        "zr_shading_toon.wgsl",
        "zr_gbuffer_encode_toon.wgsl",
        "zr_shade_deferred_toon.wgsl",
        GBufferChannelMask::standard_lit(),
    )
}

#[test]
fn render_shader_template_assembles_static_and_skinned_geometry_sources() {
    let static_mesh = static_mesh_descriptor();
    let skinned_mesh = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_SKINNED_MESH)
        .expect("skinned descriptor");

    let static_assembly = assemble_material_shader_template(material_template_request(
        static_mesh,
        ShaderPassType::Forward,
    ))
    .expect("static template assembly");
    let skinned_assembly = assemble_material_shader_template(material_template_request(
        skinned_mesh,
        ShaderPassType::Forward,
    ))
    .expect("skinned template assembly");

    assert_ne!(static_assembly.wgsl_source, skinned_assembly.wgsl_source);
    assert_include_token!(static_assembly, "zr_geometry_static.wgsl");
    assert_include_token!(static_assembly, "zr_scene_runtime.wgsl");
    assert_include_token!(static_assembly, "zr_gpu_scene.wgsl");
    assert_include_token!(static_assembly, "zr_environment.wgsl");
    assert_missing_include_token!(static_assembly, "zr_oit.wgsl");
    assert_include_token!(skinned_assembly, "zr_geometry_skinned.wgsl");
    assert!(
        static_assembly
            .wgsl_source
            .contains("fn zr_material_surface(")
    );
    assert!(static_assembly.wgsl_source.contains("fn zr_vs_main_impl("));
    assert!(static_assembly.wgsl_source.contains("fn zr_fs_main_impl("));
    assert!(static_assembly.wgsl_source.contains("fn zr_vs_main("));
    assert!(static_assembly.wgsl_source.contains("fn zr_fs_main("));
    assert!(static_assembly.wgsl_source.contains("fn vs_main("));
    assert!(static_assembly.wgsl_source.contains("fn fs_main("));
    assert!(!static_assembly.wgsl_source.contains("fn fs_oit("));
    assert!(!static_assembly.wgsl_source.contains("oit_draw("));
    assert!(
        static_assembly
            .wgsl_source
            .contains("return zr_vs_main_impl(v, instance_index);")
    );
    assert!(
        static_assembly
            .wgsl_source
            .contains("return zr_fs_main_impl(input, front_facing);")
    );
    assert!(
        static_assembly
            .wgsl_source
            .contains("@builtin(front_facing) front_facing: bool")
    );
    assert!(
        static_assembly
            .wgsl_source
            .contains("zr_surface_apply_raster_facing(zr_material_surface(input), front_facing)")
    );
    assert!(
        static_assembly
            .wgsl_source
            .contains("ZR_GEOMETRY_SOURCE_STATIC_MESH")
    );
    assert!(
        static_assembly
            .wgsl_source
            .contains("@group(0) @binding(0) var<uniform> scene: SceneUniform")
    );
    assert!(
        static_assembly
            .wgsl_source
            .contains("@group(3) @binding(1) var<storage, read> zr_instance_data")
    );
    assert!(
        static_assembly
            .wgsl_source
            .contains("let instance = zr_gpu_scene_instance(instance_index);")
    );
    assert!(
        static_assembly
            .wgsl_source
            .contains("let world_from_local = instance.world_from_local;")
    );
    assert!(
        static_assembly
            .wgsl_source
            .contains("let instance_flags = instance.flags;")
    );
    assert!(
        static_assembly
            .wgsl_source
            .contains("output.clip_position = scene.view_proj * position_ws;")
    );
    assert!(
        skinned_assembly
            .wgsl_source
            .contains("zr_skinned_joint_matrix(v.joints.x)")
    );
    assert!(
        skinned_assembly
            .wgsl_source
            .contains("@group(3) @binding(3) var<storage, read> zr_skinned_joint_palette")
    );
    assert!(
        !skinned_assembly
            .wgsl_source
            .contains("@group(3) @binding(1) var<storage, read> zr_joint_palette")
    );
    validate_material_shader_template_wgsl(&static_assembly.wgsl_source)
        .expect("static template WGSL should validate");
    validate_material_shader_template_wgsl(&skinned_assembly.wgsl_source)
        .expect("skinned template WGSL should validate");
    assert_eq!(
        static_assembly.include_content_hashes.len(),
        static_assembly.include_tokens.len()
    );
    assert_eq!(static_assembly.template_revision, "zr-material-template-v1");
}

#[test]
fn shaded_fragment_templates_apply_one_shared_double_sided_surface_frame() {
    let surface_types = include_str!("../../wgsl/zr_surface_types.wgsl");
    assert!(surface_types.contains("fn zr_surface_apply_raster_facing("));
    assert!(surface_types.contains("oriented.normal_ws = surface.normal_ws * facing_sign;"));
    assert!(surface_types.contains("oriented.bitangent_ws = surface.bitangent_ws * facing_sign;"));
    assert!(
        surface_types
            .contains("oriented.clearcoat_normal_ws = surface.clearcoat_normal_ws * facing_sign;")
    );
    assert!(!surface_types.contains("ZR_GPU_INSTANCE_FLAG_NEGATIVE_DETERMINANT"));

    for template in [
        include_str!("../../wgsl/zr_template_forward.wgsl"),
        include_str!("../../wgsl/zr_template_deferred_gbuffer.wgsl"),
        include_str!("../../wgsl/zr_template_forward_environment_only_pbr.wgsl"),
        include_str!("../../wgsl/zr_template_gbuffer.wgsl"),
    ] {
        assert!(template.contains("@builtin(front_facing) front_facing: bool"));
        assert!(
            template.contains(
                "zr_surface_apply_raster_facing(zr_material_surface(input), front_facing)"
            )
        );
    }
}

#[test]
fn render_skinned_geometry_normalizes_joint_weights_before_blending_matrices() {
    for geometry_source_id in [
        GEOMETRY_SOURCE_ID_SKINNED_MESH,
        GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH,
    ] {
        let geometry_source = builtin_geometry_source_descriptor(geometry_source_id)
            .expect("skinned geometry source descriptor");
        let assembly = assemble_material_shader_template(material_template_request(
            geometry_source,
            ShaderPassType::Forward,
        ))
        .expect("skinned template assembly");

        assert!(
            assembly
                .wgsl_source
                .contains("let inverse_weight_sum = 1.0 / weight_sum;"),
            "{geometry_source_id} should normalize valid joint weights before matrix blending"
        );
        for component in ["x", "y", "z", "w"] {
            let expected = format!(
                "zr_template_skin_weight(v.joints.{component}, v.weights.{component}, joint_count) * inverse_weight_sum"
            );
            assert!(
                assembly.wgsl_source.contains(&expected),
                "{geometry_source_id} should use normalized joint weight `{component}`"
            );
        }
    }
}

#[test]
fn render_volumetric_forward_shader_variant_removes_bindings_when_disabled() {
    let disabled = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("disabled volumetric forward variant");
    let enabled = assemble_material_shader_template(
        material_template_request(static_mesh_descriptor(), ShaderPassType::Forward)
            .with_features(ShaderFeatureBits::new(ShaderFeatureBits::VOLUMETRIC_FOG)),
    )
    .expect("enabled volumetric forward variant");

    for binding in ["@binding(25)", "@binding(26)", "@binding(27)"] {
        assert!(!disabled.wgsl_source.contains(binding));
        assert!(enabled.wgsl_source.contains(binding));
    }
    assert!(
        disabled
            .wgsl_source
            .contains("const ZR_FEATURE_VOLUMETRIC_FOG: bool = false;")
    );
    assert!(
        enabled
            .wgsl_source
            .contains("const ZR_FEATURE_VOLUMETRIC_FOG: bool = true;")
    );
    validate_material_shader_template_wgsl(&disabled.wgsl_source)
        .expect("disabled volumetric forward WGSL should validate");
    validate_material_shader_template_wgsl(&enabled.wgsl_source)
        .expect("enabled volumetric forward WGSL should validate");
}

#[test]
fn render_transmission_validates_viewport_local_projection_for_nonzero_viewport_origins() {
    let assembly = assemble_material_shader_template(
        material_template_request(static_mesh_descriptor(), ShaderPassType::Forward)
            .with_features(ShaderFeatureBits::new(ShaderFeatureBits::PBR_TRANSMISSION)),
    )
    .expect("transmission forward variant");

    for expected in [
        "struct ZrPbrViewportProjection {",
        "fn zr_pbr_viewport_projection(world_position: vec3<f32>)",
        "scene.view_proj * vec4<f32>(world_position, 1.0)",
        "if (clip_position.w <= ZR_PBR_EXTRAS_EPSILON) {",
        "let valid = all(abs(ndc.xy) <= vec2<f32>(1.0))",
        "zr_pbr_viewport_projection(transmission_frame.exit_position)",
        "ctx.position_ws",
    ] {
        assert!(
            assembly.wgsl_source.contains(expected),
            "transmission source should contain viewport-local contract `{expected}`"
        );
    }
    assert!(
        !assembly
            .wgsl_source
            .contains("fragment_position / max(transmission_extent")
    );
    assert!(
        !assembly
            .wgsl_source
            .contains("textureDimensions(zr_transmission_scene_color)")
    );
    assert!(!assembly.wgsl_source.contains("let safe_w ="));
    assert!(!assembly.wgsl_source.contains(
        "zr_pbr_screen_space_transmission(\n        surface,\n        ctx.frag_coord.xy"
    ));
    validate_material_shader_template_wgsl(&assembly.wgsl_source)
        .expect("viewport-local transmission WGSL should validate");
}

#[test]
fn render_lightmaps_share_metallic_diffuse_energy_across_template_paths() {
    let forward = include_str!("../../wgsl/zr_template_forward.wgsl");
    let deferred = include_str!("../../wgsl/zr_template_deferred_gbuffer.wgsl");
    let fallback = include_str!("../../../scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl");

    for (label, source) in [("forward", forward), ("deferred", deferred)] {
        assert!(
            source.contains("zr_surface_metallic_diffuse_energy_scale(surface.metallic),"),
            "{label} lightmap path must use the shared metallic diffuse-energy owner"
        );
        assert!(!source.contains("zr_pbr_view_diffuse_energy_scale("));
    }
    assert!(fallback.contains("zr_surface_metallic_diffuse_energy_scale(material.metallic),"));
    assert!(!fallback.contains("zr_pbr_diffuse_energy_scale_normalized_view("));
}

#[test]
fn render_shader_template_validates_morphed_geometry_sources_with_payload_slots() {
    for (geometry_source_id, include_token, source_define) in [
        (
            GEOMETRY_SOURCE_ID_MORPHED_MESH,
            "zr_geometry_morphed.wgsl",
            "ZR_GEOMETRY_SOURCE_MORPHED_MESH",
        ),
        (
            GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH,
            "zr_geometry_skinned_morphed.wgsl",
            "ZR_GEOMETRY_SOURCE_SKINNED_MORPHED_MESH",
        ),
    ] {
        let geometry_source = builtin_geometry_source_descriptor(geometry_source_id)
            .expect("morphed geometry source descriptor");

        for pass_type in [
            ShaderPassType::Forward,
            ShaderPassType::GBuffer,
            ShaderPassType::DepthPrepass,
            ShaderPassType::Shadow,
            ShaderPassType::Velocity,
            ShaderPassType::TaaReactiveMask,
            ShaderPassType::HitProxy,
        ] {
            let assembly = match pass_type {
                ShaderPassType::TaaReactiveMask => {
                    let material = standard_material_descriptor();
                    let surface_source = standard_material_surface_source(&material);
                    assemble_material_shader_template(
                        MaterialShaderTemplateRequest::new(
                            geometry_source.clone(),
                            pass_type,
                            surface_source.source.clone(),
                            surface_source.entry_point,
                        )
                        .with_features(surface_source.features),
                    )
                }
                ShaderPassType::Forward
                | ShaderPassType::GBuffer
                | ShaderPassType::DepthPrepass
                | ShaderPassType::Shadow
                | ShaderPassType::Velocity
                | ShaderPassType::HitProxy => assemble_material_shader_template(
                    material_template_request(geometry_source.clone(), pass_type),
                ),
            }
            .expect("morphed template assembly");

            assert_include_token!(assembly, include_token);
            assert!(assembly.wgsl_source.contains(source_define));
            assert!(
                assembly
                    .wgsl_source
                    .contains("@builtin(vertex_index) vertex_index: u32")
            );
            assert!(assembly.wgsl_source.contains("morph_payload_slot"));
            assert!(assembly.wgsl_source.contains("zr_gpu_scene_morph_payload"));
            assert!(assembly.wgsl_source.contains("zr_morph_previous_weight"));
            assert!(
                assembly
                    .wgsl_source
                    .contains("payload.y + payload.w + target_index")
            );
            validate_material_shader_template_wgsl(&assembly.wgsl_source)
                .expect("morphed template WGSL should validate");
        }
    }
}

#[test]
fn standard_material_surface_source_can_be_built_from_runtime_features() {
    let features = ShaderFeatureBits::new(
        ShaderFeatureBits::ALPHA_TEST
            | ShaderFeatureBits::RECEIVE_SHADOWS
            | ShaderFeatureBits::DOUBLE_SIDED,
    );
    let surface_source = standard_material_surface_source_for_features(features, 1.25);

    assert_eq!(surface_source.entry_point, "standard_material_surface");
    assert_eq!(surface_source.features, features);
    assert!(
        surface_source
            .source
            .contains("const ZR_STANDARD_MATERIAL_ALPHA_CUTOFF: f32 = 1.00000000;")
    );
    assert!(
        surface_source
            .features
            .contains(ShaderFeatureBits::RECEIVE_SHADOWS)
    );

    let nan_surface_source =
        standard_material_surface_source_for_features(ShaderFeatureBits::default(), f32::NAN);
    assert!(
        nan_surface_source
            .source
            .contains("const ZR_STANDARD_MATERIAL_ALPHA_CUTOFF: f32 = 0.00000000;")
    );
}

#[test]
fn render_shader_template_validates_standard_material_wgsl_with_naga() {
    let static_mesh = static_mesh_descriptor();
    let material = standard_material_descriptor();
    let surface_source = standard_material_surface_source(&material);
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

    let validation = validate_material_shader_template_wgsl(&assembly.wgsl_source)
        .expect("assembled standard material WGSL should validate");

    for expected in [
        "// include: zr_lightmap.wgsl",
        "@group(1) @binding(23) var<storage, read> zr_light_probe_grid",
        "@group(1) @binding(24) var zr_lightmap_atlas: texture_2d_array<f32>;",
        "@group(1) @binding(28) var zr_lightmap_sampler: sampler;",
        "zr_lightmap_baked_irradiance(",
    ] {
        assert!(assembly.wgsl_source.contains(expected));
    }

    for expected in ["zr_vs_main", "vs_main", "zr_fs_main", "fs_main"] {
        assert!(
            validation
                .entry_points
                .iter()
                .any(|entry_point| entry_point == expected),
            "assembled standard material WGSL should expose `{expected}`"
        );
    }
}

#[test]
fn standard_pbr_clearcoat_base_energy_variants_validate_with_naga() {
    let clearcoat_disabled = standard_material_descriptor();
    let mut clearcoat_enabled = standard_material_descriptor();
    clearcoat_enabled.advanced_features.clearcoat = 0.75;
    let mut blinn_phong = standard_material_descriptor();
    blinn_phong.lighting_model = RenderMaterialLightingModel::BlinnPhong;

    for (label, material) in [
        ("standard PBR clearcoat disabled", clearcoat_disabled),
        ("standard PBR clearcoat enabled", clearcoat_enabled),
        ("Blinn-Phong", blinn_phong),
    ] {
        let surface_source = standard_material_surface_source(&material);
        let assembly = assemble_material_shader_template(
            MaterialShaderTemplateRequest::new(
                static_mesh_descriptor(),
                ShaderPassType::Forward,
                surface_source.source,
                surface_source.entry_point,
            )
            .with_features(surface_source.features),
        )
        .unwrap_or_else(|error| panic!("{label} template should assemble: {error:?}"));

        validate_material_shader_template_wgsl(&assembly.wgsl_source)
            .unwrap_or_else(|error| panic!("{label} composed WGSL should validate: {error:?}"));
    }
}

#[test]
fn render_deferred_gbuffer_template_validates_baked_indirect_output() {
    let material = standard_material_descriptor();
    let surface_source = standard_material_surface_source(&material);
    let assembly = assemble_deferred_gbuffer_shader_template(
        DeferredGBufferShaderTemplateRequest::new(
            static_mesh_descriptor(),
            surface_source.source,
            surface_source.entry_point,
        )
        .with_features(surface_source.features),
    )
    .expect("standard deferred GBuffer template assembly");

    for expected in [
        "// include: zr_lightmap.wgsl",
        "// include: zr_pbr_common.wgsl",
        "let baked_diffuse_color = surface.base_color.rgb;",
        "baked_diffuse_color = zr_pbr_base_color(surface.base_color.rgb);",
        "let baked_indirect = baked_diffuse_color * diffuse_energy_scale",
        "zr_surface_metallic_diffuse_energy_scale(surface.metallic),",
        "output.emissive = vec4<f32>(output.emissive.rgb + baked_indirect",
    ] {
        assert!(assembly.wgsl_source.contains(expected));
    }
    validate_material_shader_template_wgsl(&assembly.wgsl_source)
        .expect("deferred GBuffer lightmap WGSL should validate");
}

#[test]
fn standard_pbr_templates_apply_metallic_diffuse_energy_to_baked_indirect() {
    let material = standard_material_descriptor();
    let surface_source = standard_material_surface_source(&material);
    let forward = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            static_mesh_descriptor(),
            ShaderPassType::Forward,
            surface_source.source.clone(),
            surface_source.entry_point,
        )
        .with_features(surface_source.features),
    )
    .expect("standard forward template assembly");
    let deferred = assemble_deferred_gbuffer_shader_template(
        DeferredGBufferShaderTemplateRequest::new(
            static_mesh_descriptor(),
            surface_source.source,
            surface_source.entry_point,
        )
        .with_features(surface_source.features),
    )
    .expect("standard deferred GBuffer template assembly");

    for source in [&forward.wgsl_source, &deferred.wgsl_source] {
        assert!(source.contains("let baked_diffuse_color = surface.base_color.rgb;"));
        assert!(
            source.contains("baked_diffuse_color = zr_pbr_base_color(surface.base_color.rgb);")
        );
        assert!(source.contains("zr_surface_metallic_diffuse_energy_scale(surface.metallic),"));
        assert!(!source.contains("zr_pbr_view_diffuse_energy_scale("));
        assert!(!source.contains("zr_pbr_diffuse_energy_scale("));
        assert!(!source.contains("metallic * 0.45"));
    }
}

#[test]
fn render_shader_template_clips_alpha_for_masked_standard_material_passes() {
    let static_mesh = static_mesh_descriptor();
    let material = standard_material_descriptor();
    let surface_source = standard_material_surface_source(&material);

    let depth_alpha = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            static_mesh.clone(),
            ShaderPassType::DepthPrepass,
            surface_source.source.clone(),
            surface_source.entry_point,
        )
        .with_features(surface_source.features),
    )
    .expect("alpha depth template assembly");
    let shadow_alpha = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            static_mesh,
            ShaderPassType::Shadow,
            surface_source.source,
            surface_source.entry_point,
        )
        .with_features(surface_source.features),
    )
    .expect("alpha shadow template assembly");

    for source in [&depth_alpha.wgsl_source, &shadow_alpha.wgsl_source] {
        assert!(source.contains("@fragment"));
        assert!(source.contains("let surface = zr_material_surface(input);"));
        assert!(source.contains("zr_apply_alpha_clip(surface);"));
        assert!(source.contains("fn zr_apply_alpha_clip(surface: ZrSurfaceOutput)"));
        assert!(source.contains("surface.alpha_cutoff = standard_material_alpha_cutoff();"));
        assert!(source.contains("standard_material_properties.data8.z"));
        assert!(source.contains("const ZR_STANDARD_MATERIAL_ALPHA_CUTOFF: f32 = 0.50000000;"));
        assert!(source.contains("const ZR_FEATURE_ALPHA_TEST: bool = true;"));
    }
    assert_include_token!(depth_alpha, "zr_template_depth_alpha.wgsl");
    assert_include_token!(shadow_alpha, "zr_template_shadow_alpha.wgsl");
}

#[test]
fn render_shader_template_specializes_depth_and_velocity_passes() {
    let static_mesh = static_mesh_descriptor();

    let depth_no_alpha = assemble_material_shader_template(material_template_request(
        static_mesh.clone(),
        ShaderPassType::DepthPrepass,
    ))
    .expect("depth template assembly");
    let depth_alpha = assemble_material_shader_template(
        material_template_request(static_mesh.clone(), ShaderPassType::DepthPrepass)
            .with_features(ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST)),
    )
    .expect("alpha depth template assembly");
    let velocity = assemble_material_shader_template(material_template_request(
        static_mesh.clone(),
        ShaderPassType::Velocity,
    ))
    .expect("velocity template assembly");
    let velocity_alpha = assemble_material_shader_template(
        material_template_request(static_mesh, ShaderPassType::Velocity)
            .with_features(ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST)),
    )
    .expect("alpha velocity template assembly");

    assert_include_token!(depth_no_alpha, "zr_template_depth.wgsl");
    assert!(!depth_no_alpha.wgsl_source.contains("zr_material_surface"));
    assert!(!depth_no_alpha.wgsl_source.contains("@fragment"));
    assert!(
        !depth_no_alpha
            .wgsl_source
            .contains("surface.normal_ws * 0.5")
    );
    assert!(
        !depth_no_alpha
            .wgsl_source
            .contains("zr_template_gbuffer.wgsl")
    );
    assert!(depth_alpha.wgsl_source.contains("zr_material_surface"));
    assert_include_token!(depth_alpha, "zr_template_depth_alpha.wgsl");
    assert!(
        depth_alpha
            .wgsl_source
            .contains("zr_apply_alpha_clip(surface);")
    );
    assert!(!depth_alpha.wgsl_source.contains("surface.normal_ws * 0.5"));
    assert!(!depth_alpha.wgsl_source.contains("zr_template_gbuffer.wgsl"));
    assert!(velocity.wgsl_source.contains("fetch_prev_position"));
    assert!(
        velocity
            .wgsl_source
            .contains("struct ZrVelocityVertexInput")
    );
    assert!(
        velocity
            .wgsl_source
            .contains("@location(8) previous_position")
    );
    assert!(
        velocity
            .wgsl_source
            .contains("let previous_input = zr_velocity_vertex_input(v, v.previous_position);")
    );
    assert!(velocity.wgsl_source.contains("fn zr_vs_main_impl("));
    assert!(velocity.wgsl_source.contains("fn zr_fs_main_impl("));
    assert!(velocity.wgsl_source.contains("fn vs_main("));
    assert!(velocity.wgsl_source.contains("fn fs_main("));
    assert!(
        velocity
            .wgsl_source
            .contains("scene.view_proj_unjittered * current_world")
    );
    assert!(
        velocity
            .wgsl_source
            .contains("scene.previous_view_proj_unjittered * previous_world")
    );
    assert!(
        velocity
            .wgsl_source
            .contains("return zr_vs_main_impl(v, instance_index);")
    );
    assert!(!velocity.wgsl_source.contains("zr_material_surface"));
    assert!(velocity_alpha.wgsl_source.contains("zr_material_surface"));
    assert!(
        velocity_alpha
            .wgsl_source
            .contains("zr_surface_fails_alpha_clip(surface)")
    );
    assert_include_token!(velocity_alpha, "zr_template_velocity_alpha.wgsl");
    validate_material_shader_template_wgsl(&depth_no_alpha.wgsl_source)
        .expect("assembled depth-only WGSL should validate");
    validate_material_shader_template_wgsl(&depth_alpha.wgsl_source)
        .expect("assembled alpha depth-only WGSL should validate");
    validate_material_shader_template_wgsl(&velocity.wgsl_source)
        .expect("assembled velocity WGSL should validate");
    validate_material_shader_template_wgsl(&velocity_alpha.wgsl_source)
        .expect("assembled alpha velocity WGSL should validate");

    let velocity_pass = pass_template_for(ShaderPassType::Velocity, ShaderFeatureBits::default());
    let velocity_alpha_pass = pass_template_for(
        ShaderPassType::Velocity,
        ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST),
    );
    assert!(velocity_pass.uses_previous_position);
    assert!(!velocity_pass.requires_material_surface);
    assert!(velocity_alpha_pass.uses_previous_position);
    assert!(velocity_alpha_pass.requires_material_surface);
}

#[test]
fn render_shader_template_specializes_hit_proxy_identity_and_geometry_products() {
    let static_mesh = static_mesh_descriptor();
    let opaque = assemble_material_shader_template(material_template_request(
        static_mesh.clone(),
        ShaderPassType::HitProxy,
    ))
    .expect("opaque HitProxy template assembly");
    let alpha = assemble_material_shader_template(
        material_template_request(static_mesh, ShaderPassType::HitProxy)
            .with_features(ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST)),
    )
    .expect("alpha HitProxy template assembly");

    assert_include_token!(opaque, "zr_template_hit_proxy.wgsl");
    assert_include_token!(alpha, "zr_template_hit_proxy_alpha.wgsl");
    for assembly in [&opaque, &alpha] {
        assert!(assembly.wgsl_source.contains("@location(0) token: u32"));
        assert!(
            assembly
                .wgsl_source
                .contains("@location(1) world_position_depth: vec4<f32>")
        );
        assert!(
            assembly
                .wgsl_source
                .contains("@location(2) world_normal: vec4<f32>")
        );
        assert!(
            assembly
                .wgsl_source
                .contains("zr_hit_proxy_token(input.instance_index)")
        );
        validate_material_shader_template_wgsl(&assembly.wgsl_source)
            .expect("assembled HitProxy WGSL should validate");
    }
    assert!(!opaque.wgsl_source.contains("zr_material_surface"));
    assert!(alpha.wgsl_source.contains("zr_material_surface"));
    assert!(
        alpha
            .wgsl_source
            .contains("zr_surface_fails_alpha_clip(surface)")
    );
}

#[test]
fn render_shader_template_uses_shading_model_descriptor_forward_include() {
    let static_mesh = static_mesh_descriptor();
    let descriptor = ShadingModelDescriptor::new(
        SHADING_MODEL_ID_STANDARD_PBR,
        "standard_pbr",
        "zr_shading_standard_pbr.wgsl",
        "zr_gbuffer_encode_standard_pbr.wgsl",
        "zr_shade_deferred_standard_pbr.wgsl",
        GBufferChannelMask::standard_lit(),
    );

    let assembly = assemble_material_shader_template(
        material_template_request(static_mesh, ShaderPassType::Forward)
            .with_shading_model_descriptor(descriptor),
    )
    .expect("descriptor-backed forward template assembly");

    assert_include_token!(assembly, "zr_shading_standard_pbr.wgsl");
    assert_include_token!(assembly, "zr_environment.wgsl");
    assert_eq!(
        assembly
            .include_tokens
            .iter()
            .filter(|token| token.as_str() == "zr_shading_standard_pbr.wgsl")
            .count(),
        1
    );
    assert!(
        assembly
            .wgsl_source
            .contains("// include: zr_shading_standard_pbr.wgsl")
    );
    assert!(
        assembly
            .wgsl_source
            .contains("fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext)")
    );
}

#[test]
fn render_shader_template_rejects_unknown_shading_model_forward_include() {
    let static_mesh = static_mesh_descriptor();
    let descriptor = toon_shading_model_descriptor();

    let error = assemble_material_shader_template(
        material_template_request(static_mesh, ShaderPassType::Forward)
            .with_shading_model_descriptor(descriptor),
    )
    .expect_err("unknown descriptor include should fail before template assembly succeeds");

    assert_eq!(
        error,
        ShaderTemplateAssemblyError::UnknownShadingInclude {
            token: "zr_shading_toon.wgsl".to_string(),
        }
    );
}

#[test]
fn render_shader_template_uses_custom_shading_model_forward_include_source() {
    let static_mesh = static_mesh_descriptor();
    let descriptor = toon_shading_model_descriptor();

    let assembly = assemble_material_shader_template(
        material_template_request(static_mesh, ShaderPassType::Forward)
            .with_shading_model_descriptor(descriptor)
            .with_shading_model_forward_include_source(
                "zr_shading_toon.wgsl",
                CUSTOM_TOON_FORWARD_INCLUDE,
            ),
    )
    .expect("custom descriptor-backed forward template assembly");

    assert_include_token!(assembly, "zr_shading_toon.wgsl");
    assert_missing_include_token!(assembly, "zr_shading_standard_pbr.wgsl");
    assert!(
        assembly
            .wgsl_source
            .contains("// include: zr_shading_toon.wgsl")
    );
    assert!(assembly.wgsl_source.contains("ZR_SHADING_TOON_DEBUG_ID"));
    assert!(assembly.wgsl_source.contains("fn zr_toon_band"));
    assert!(
        assembly
            .wgsl_source
            .contains("fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext)")
    );
    assert_include_token!(assembly, "zr_environment.wgsl");
    validate_material_shader_template_wgsl(&assembly.wgsl_source)
        .expect("custom shading include template WGSL should validate");
}

#[test]
fn render_deferred_gbuffer_template_rejects_unknown_shading_model_gbuffer_include() {
    let static_mesh = static_mesh_descriptor();
    let descriptor = toon_shading_model_descriptor();

    let error = assemble_deferred_gbuffer_shader_template(
        DeferredGBufferShaderTemplateRequest::new(static_mesh, MATERIAL_SURFACE, "user_surface")
            .with_shading_model_descriptor(descriptor),
    )
    .expect_err("unknown descriptor GBuffer include should fail before template assembly succeeds");

    assert_eq!(
        error,
        ShaderTemplateAssemblyError::UnknownShadingInclude {
            token: "zr_gbuffer_encode_toon.wgsl".to_string(),
        }
    );
}

#[test]
fn render_deferred_gbuffer_template_uses_custom_shading_model_gbuffer_include_source() {
    let static_mesh = static_mesh_descriptor();
    let descriptor = toon_shading_model_descriptor();

    let assembly = assemble_deferred_gbuffer_shader_template(
        DeferredGBufferShaderTemplateRequest::new(static_mesh, MATERIAL_SURFACE, "user_surface")
            .with_shading_model_descriptor(descriptor)
            .with_shading_model_gbuffer_include_source(
                "zr_gbuffer_encode_toon.wgsl",
                CUSTOM_TOON_GBUFFER_INCLUDE,
            ),
    )
    .expect("custom descriptor-backed deferred GBuffer template assembly");

    assert_include_token!(assembly, "zr_gbuffer_encode_toon.wgsl");
    assert_missing_include_token!(assembly, "zr_gbuffer_encode_standard_pbr.wgsl");
    assert!(
        assembly
            .wgsl_source
            .contains("// include: zr_gbuffer_encode_toon.wgsl")
    );
    assert!(assembly.wgsl_source.contains("ZR_GBUFFER_TOON_DEBUG_ID"));
    assert!(assembly.wgsl_source.contains("fn encode_gbuffer"));
    assert!(
        assembly
            .wgsl_source
            .contains("// include: zr_template_deferred_gbuffer.wgsl")
    );
    validate_material_shader_template_wgsl(&assembly.wgsl_source)
        .expect("custom deferred GBuffer include template WGSL should validate");
}

#[test]
fn render_material_templates_route_ambient_by_lightmap_presence() {
    let forward = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("forward material template assembly");
    let deferred =
        assemble_deferred_gbuffer_shader_template(DeferredGBufferShaderTemplateRequest::new(
            static_mesh_descriptor(),
            MATERIAL_SURFACE,
            "user_surface",
        ))
        .expect("deferred GBuffer template assembly");

    assert!(forward.wgsl_source.contains("zr_scene_ambient_color("));
    assert!(
        forward
            .wgsl_source
            .contains("zr_gpu_scene_has_lightmap(ctx.instance_index)")
    );
    assert!(deferred.wgsl_source.contains("output.emissive.a = select("));
    assert!(
        deferred
            .wgsl_source
            .contains("zr_gpu_scene_has_lightmap(input.instance_index)")
    );
}
