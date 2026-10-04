use crate::core::framework::render::{
    builtin_geometry_source_descriptor, ShaderPassType, GENERATED_MATERIAL_MODULE_IMPORT_PATH,
    GEOMETRY_SOURCE_ID_STATIC_MESH,
};

use super::{
    assemble_material_shader_template, shader_assembly_source_location_for_line,
    MaterialShaderTemplateRequest, ShaderAssemblySegmentKind, MATERIAL_SURFACE_ENTRY_POINT,
};

const GENERATED_MATERIAL: &str = r#"
fn generated_material_value() -> vec4<f32> {
    return vec4<f32>(0.2, 0.4, 0.8, 1.0);
}
"#;

const USER_SURFACE: &str = r#"
#include <self::material>

fn user_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    return zr_surface_from_base_color(generated_material_value() + input.color * 0.0);
}
"#;

#[test]
fn shader_template_assembly_records_source_segments_for_diagnostics() {
    let geometry_source = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_STATIC_MESH)
        .expect("static geometry source");
    let assembly = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            geometry_source,
            ShaderPassType::Forward,
            USER_SURFACE,
            "user_surface",
        )
        .with_generated_material_source(GENERATED_MATERIAL)
        .with_material_surface_module_id("project::materials::hero"),
    )
    .expect("template assembly");

    let generated_segment = assembly
        .segments
        .iter()
        .find(|segment| segment.module_id == GENERATED_MATERIAL_MODULE_IMPORT_PATH)
        .expect("generated material segment");
    let generated_source_line = generated_segment.assembled_start_line + 1;
    let generated_location =
        shader_assembly_source_location_for_line(&assembly.segments, generated_source_line)
            .expect("generated material source location");
    assert_eq!(
        generated_location.module_id,
        GENERATED_MATERIAL_MODULE_IMPORT_PATH
    );
    assert_eq!(
        generated_location.kind,
        ShaderAssemblySegmentKind::GeneratedMaterial
    );
    assert_eq!(generated_location.local_line, 1);

    let surface_line = assembly
        .wgsl_source
        .lines()
        .position(|line| line.contains(MATERIAL_SURFACE_ENTRY_POINT))
        .expect("renamed material surface line") as u32
        + 1;
    let surface_location =
        shader_assembly_source_location_for_line(&assembly.segments, surface_line)
            .expect("surface source location");
    assert_eq!(surface_location.module_id, "project::materials::hero");
    assert_eq!(
        surface_location.kind,
        ShaderAssemblySegmentKind::UserMaterialSurface
    );
    assert_eq!(surface_location.local_line, 2);
}

#[test]
fn all_material_template_domains_publish_the_same_assembly_profile_stage() {
    let forward = include_str!("../assemble.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("forward template test boundary");
    let deferred = include_str!("../deferred_gbuffer.rs");
    let taa = include_str!("../taa_reactive_mask.rs");

    for source in [forward, deferred, taa] {
        assert!(source.contains("\"shader_pipeline\", \"template_assembly\""));
    }
}
