use super::*;

fn include_content_hash<'a>(
    assembly: &'a super::super::assemble::MaterialShaderTemplateAssembly,
    token: &str,
) -> &'a str {
    assembly
        .include_tokens
        .iter()
        .zip(&assembly.include_content_hashes)
        .find_map(|(include_token, hash)| (include_token == token).then_some(hash.as_str()))
        .unwrap_or_else(|| panic!("Forward template should retain canonical `{token}` include"))
}

fn standard_pbr_forward_request(features: ShaderFeatureBits) -> MaterialShaderTemplateRequest {
    let surface_source = standard_material_surface_source_for_features(features, 0.5);
    MaterialShaderTemplateRequest::new(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
        surface_source.source,
        surface_source.entry_point,
    )
    .with_features(surface_source.features)
}

#[test]
fn forward_base_pbr_specializes_advanced_source_out_of_its_compilation_closure() {
    let basic = assemble_material_shader_template(standard_pbr_forward_request(
        ShaderFeatureBits::default(),
    ))
    .expect("basic Standard-PBR Forward template assembly");
    let advanced_variants = [
        ("clearcoat", ShaderFeatureBits::PBR_CLEARCOAT),
        ("anisotropy", ShaderFeatureBits::PBR_ANISOTROPY),
        ("transmission", ShaderFeatureBits::PBR_TRANSMISSION),
    ]
    .map(|(label, feature)| {
        let assembly = assemble_material_shader_template(standard_pbr_forward_request(
            ShaderFeatureBits::new(feature),
        ))
        .unwrap_or_else(|error| panic!("{label} Standard-PBR Forward template assembly: {error}"));
        (label, assembly)
    });

    for assembly in [&basic]
        .into_iter()
        .chain(advanced_variants.iter().map(|(_, assembly)| assembly))
    {
        assert_include_token!(assembly, "zr_pbr_extras.wgsl");
        assert_include_token!(assembly, "zr_shading_standard_pbr.wgsl");
        for required in [
            "fn zr_pbr_fresnel_schlick(",
            "fn zr_pbr_smith_visibility(",
            "fn zr_pbr_isotropic_ggx(",
            "fn zr_standard_pbr_gpu_light_lighting(",
            "zr_environment_pbr_indirect_normalized(",
            "zr_gpu_light_shadow_visibility(",
        ] {
            assert!(
                assembly.wgsl_source.contains(required),
                "Forward PBR must retain its base closure `{required}`"
            );
        }
        validate_material_shader_template_wgsl(&assembly.wgsl_source)
            .expect("specialized Forward PBR WGSL should validate");
    }

    for excluded in [
        "@group(1) @binding(31) var zr_transmission_scene_color",
        "@group(1) @binding(32) var zr_transmission_scene_color_sampler",
        "fn zr_pbr_rotated_tangent(",
        "fn zr_aniso_ggx(",
        "fn zr_clearcoat_lobe(",
        "fn zr_pbr_advanced_environment(",
        "fn zr_pbr_screen_space_transmission(",
        "if (ZR_FEATURE_PBR_ANISOTROPY)",
        "if (ZR_FEATURE_PBR_CLEARCOAT",
        "if (ZR_FEATURE_PBR_TRANSMISSION",
    ] {
        assert!(
            !basic.wgsl_source.contains(excluded),
            "basic Forward PBR must exclude unreachable advanced source `{excluded}`"
        );
        assert!(
            advanced_variants
                .iter()
                .all(|(_, assembly)| assembly.wgsl_source.contains(excluded)),
            "every advanced Forward PBR variant must retain `{excluded}`"
        );
    }

    for token in ["zr_pbr_extras.wgsl", "zr_shading_standard_pbr.wgsl"] {
        for (label, advanced) in &advanced_variants {
            assert_ne!(
                include_content_hash(&basic, token),
                include_content_hash(advanced, token),
                "basic and {label} Forward PBR must separate `{token}` cache content"
            );
        }
    }
    for (label, advanced) in &advanced_variants {
        assert!(
            basic.wgsl_source.len() < advanced.wgsl_source.len(),
            "basic Forward PBR must compile less source than {label}, basic={} advanced={}",
            basic.wgsl_source.len(),
            advanced.wgsl_source.len(),
        );
    }
}

#[test]
fn custom_surface_without_a_shading_descriptor_keeps_the_full_pbr_closure() {
    let custom = assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("custom Forward template assembly");

    for required in [
        "@group(1) @binding(31) var zr_transmission_scene_color",
        "fn zr_aniso_ggx(",
        "fn zr_clearcoat_lobe(",
        "fn zr_pbr_screen_space_transmission(",
    ] {
        assert!(
            custom.wgsl_source.contains(required),
            "custom surface without an explicit descriptor must retain `{required}`"
        );
    }
    validate_material_shader_template_wgsl(&custom.wgsl_source)
        .expect("custom Forward WGSL should validate with the full PBR closure");
}

#[test]
fn custom_environment_only_surface_keeps_the_generic_forward_closure() {
    let custom = assemble_material_shader_template(
        material_template_request(static_mesh_descriptor(), ShaderPassType::Forward).with_features(
            ShaderFeatureBits::new(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR),
        ),
    )
    .expect("custom environment-only Forward template assembly");

    assert_include_token!(custom, "zr_template_forward.wgsl");
    for required in [
        "fn zr_standard_pbr_gpu_light_lighting(",
        "@group(1) @binding(31) var zr_transmission_scene_color",
        "fn zr_aniso_ggx(",
    ] {
        assert!(
            custom.wgsl_source.contains(required),
            "custom environment-only surface must retain generic Forward `{required}`"
        );
    }
    validate_material_shader_template_wgsl(&custom.wgsl_source)
        .expect("custom environment-only Forward WGSL should validate");
}
