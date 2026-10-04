use crate::core::framework::render::{
    builtin_geometry_source_descriptor, ShaderPassType, GEOMETRY_SOURCE_ID_STATIC_MESH,
};

use super::{validate_material_shader_template_assembly, ShaderTemplateValidationError};
use crate::graphics::shader::template::assemble::{
    assemble_material_shader_template, MaterialShaderTemplateRequest,
};

const INVALID_USER_SURFACE: &str = r#"
fn user_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    let bad = vec4<f32>(1.0;
    return zr_surface_from_base_color(input.color + bad);
}
"#;

#[test]
fn shader_template_validation_remaps_parse_errors_to_source_segment() {
    let geometry_source = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_STATIC_MESH)
        .expect("static geometry source");
    let assembly = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            geometry_source,
            ShaderPassType::Forward,
            INVALID_USER_SURFACE,
            "user_surface",
        )
        .with_material_surface_module_id("project::materials::invalid"),
    )
    .expect("template assembly");

    let error = validate_material_shader_template_assembly(&assembly)
        .expect_err("invalid user WGSL should fail");
    let ShaderTemplateValidationError::Parse { message } = error else {
        panic!("expected parse error");
    };
    assert!(
        message.contains("Zircon shader source: project::materials::invalid:"),
        "{message}"
    );
}
