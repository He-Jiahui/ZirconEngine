use crate::core::framework::render::ShaderPassType;

use super::environment::wgsl_function_source;
use super::{
    assemble_material_shader_template, material_template_request, static_mesh_descriptor,
    validate_material_shader_template_wgsl,
};

fn assembled_forward_source() -> String {
    assemble_material_shader_template(material_template_request(
        static_mesh_descriptor(),
        ShaderPassType::Forward,
    ))
    .expect("generic Forward template assembly")
    .wgsl_source
}

fn specular_occlusion_reference(no_v: f32, roughness: f32, occlusion: f32) -> f32 {
    let clamped_no_v = no_v.clamp(0.0, 1.0);
    let clamped_roughness = roughness.clamp(0.0, 1.0);
    let clamped_occlusion = occlusion.clamp(0.0, 1.0);
    if clamped_occlusion <= 0.0 || clamped_occlusion >= 1.0 || clamped_roughness <= 0.000001 {
        return clamped_occlusion;
    }
    let roughness_sq = clamped_roughness * clamped_roughness;
    ((clamped_no_v + clamped_occlusion).powf(roughness_sq) - 1.0 + clamped_occlusion)
        .clamp(0.0, 1.0)
}

#[test]
fn environment_specular_occlusion_forward_template_is_valid_wgsl() {
    let source = assembled_forward_source();

    validate_material_shader_template_wgsl(&source)
        .expect("environment specular occlusion Forward WGSL validation");
}

#[test]
fn environment_specular_occlusion_matches_unreal_roughness_squared_contract() {
    let source = assembled_forward_source();
    let helper = wgsl_function_source(&source, "fn zr_environment_specular_occlusion(");

    assert!(helper.contains("let roughness_sq = clamped_roughness * clamped_roughness;"));
    assert!(helper.contains("return clamped_occlusion;"));
    assert!(
        helper.contains(
            "pow(clamped_no_v + clamped_occlusion, roughness_sq) - 1.0 + clamped_occlusion"
        )
    );
    assert!(helper.contains("return clamp(specular_occlusion, 0.0, 1.0);"));
}

#[test]
fn environment_specular_occlusion_reference_preserves_boundaries_and_view_dependence() {
    assert_eq!(specular_occlusion_reference(0.2, 0.8, 0.0), 0.0);
    assert_eq!(specular_occlusion_reference(0.2, 0.8, 1.0), 1.0);
    assert_eq!(specular_occlusion_reference(0.2, 0.0, 0.35), 0.35);

    let grazing = specular_occlusion_reference(0.05, 0.8, 0.35);
    let facing = specular_occlusion_reference(0.95, 0.8, 0.35);
    assert!(grazing < facing);
    assert_ne!(grazing, 0.35);
}

#[test]
fn environment_components_apply_ao_separately_to_diffuse_and_specular() {
    let source = assembled_forward_source();
    let components =
        wgsl_function_source(&source, "fn zr_environment_pbr_components_from_reflection(");

    assert!(components.contains("diffuse_environment * clamped_occlusion"));
    assert!(components.contains(
        "let specular_occlusion = zr_environment_specular_occlusion(\n            no_v,\n            clamped_roughness,\n            clamped_occlusion,\n        );"
    ));
    assert!(components.contains("* specular_occlusion;"));
    assert!(
        components
            .contains("diffuse_environment * clamped_occlusion,\n        specular_environment,")
    );
    assert!(!components.contains("specular_environment * clamped_occlusion"));
}

#[test]
fn clearcoat_environment_uses_shared_specular_occlusion_before_texture_work() {
    let source = assembled_forward_source();
    let clearcoat = wgsl_function_source(&source, "fn zr_pbr_advanced_environment_normalized(");

    let occlusion = clearcoat
        .find("let clamped_occlusion = clamp(surface.occlusion, 0.0, 1.0);")
        .expect("clearcoat must clamp material occlusion");
    let early_out = clearcoat
        .find("if (clamped_occlusion <= 0.0) {")
        .expect("zero AO must skip clearcoat environment work");
    let planar = clearcoat
        .find("let planar = zr_environment_planar_reflection(")
        .expect("clearcoat must retain planar reflection selection");

    assert!(occlusion < early_out && early_out < planar);
    assert!(
        clearcoat.contains(
            "zr_environment_specular_occlusion(no_v, clamped_roughness, clamped_occlusion)"
        )
    );
    assert!(
        clearcoat.contains("zr_environment_env_brdf_lut(vec3<f32>(0.04), clamped_roughness, no_v)")
    );
    assert!(!clearcoat.contains("* clamp(surface.occlusion, 0.0, 1.0)"));
}

#[test]
fn fallback_mesh_uses_the_same_gltf_occlusion_strength_contract() {
    let fallback = include_str!("../../../scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl");

    assert!(
        fallback
            .contains("mix(1.0, occlusion_sample, clamp(material_properties.data0.z, 0.0, 1.0))")
    );
    assert!(!fallback.contains("occlusion * textureSampleBias("));
    assert!(!fallback.contains("if (occlusion <= 0.0)"));
}

#[test]
fn fallback_mesh_keeps_zero_gltf_roughness_at_the_pbr_minimum() {
    let fallback = include_str!("../../../scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl");

    assert!(fallback.contains(
        "let roughness = clamp(\n        material_properties.data0.y * metallic_roughness.g,\n        ZR_STANDARD_MATERIAL_MIN_ROUGHNESS,\n        1.0,\n    );"
    ));
    assert!(!fallback.contains("if (roughness <= 0.0)"));
    assert!(!fallback.contains("roughness = 1.0;"));
}
