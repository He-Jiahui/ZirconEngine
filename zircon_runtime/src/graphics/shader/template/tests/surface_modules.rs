use crate::core::framework::render::{
    RenderShaderDefinitionValue, ShaderPassType, GENERATED_MATERIAL_MODULE_IMPORT_PATH,
};

use super::super::assemble::{
    assemble_material_shader_template, MaterialShaderTemplateRequest, ShaderTemplateAssemblyError,
};
use super::super::module_registry::ShaderTemplateInclude;
use super::{has_include_token, static_mesh_descriptor};

#[test]
fn render_shader_template_expands_declared_surface_modules_and_strips_directives() {
    let static_mesh = static_mesh_descriptor();
    let source = r#"
#include <self::material>
#include <zr_surface_types.wgsl>
#include <project::cloth_math>

fn user_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    return zr_surface_from_base_color(cloth_debug_color(input.color));
}
"#;

    let assembly = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            static_mesh,
            ShaderPassType::Forward,
            source,
            "user_surface",
        )
        .with_generated_material_source(
            "fn zr_mat_detail_layer() -> bool { return ZR_OPT_DETAIL_LAYER; }",
        )
        .with_module_include_sources([ShaderTemplateInclude::new(
            "project::cloth_math",
            "fn cloth_debug_color(color: vec4<f32>) -> vec4<f32> { return color; }",
        )])
        .with_material_option_defines([RenderShaderDefinitionValue::bool(
            "ZR_OPT_DETAIL_LAYER",
            true,
        )]),
    )
    .expect("surface module import should assemble");

    assert_include_token!(assembly, "self::material");
    assert_include_token!(assembly, "zr_surface_types.wgsl");
    assert_include_token!(assembly, "project::cloth_math");
    assert_eq!(
        assembly
            .wgsl_source
            .matches("// include: zr_surface_types.wgsl")
            .count(),
        1
    );
    assert!(assembly
        .wgsl_source
        .contains("fn cloth_debug_color(color: vec4<f32>)"));
    assert!(assembly
        .wgsl_source
        .contains("const ZR_OPT_DETAIL_LAYER: bool = true;"));
    assert!(!assembly.wgsl_source.contains("#include <self::material>"));
    assert!(!assembly
        .wgsl_source
        .contains("#include <project::cloth_math>"));
    assert!(!assembly
        .wgsl_source
        .contains("#include <zr_surface_types.wgsl>"));
}

#[test]
fn render_shader_self_material_anchor_is_byte_identical_to_auto_injection() {
    let static_mesh = static_mesh_descriptor();
    let generated_material = "fn zr_mat_detail_layer() -> bool { return true; }";
    let surface_without_anchor = r#"
fn user_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    return zr_surface_from_base_color(input.color);
}
"#;
    let surface_with_anchor = r#"#include <self::material>

fn user_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    return zr_surface_from_base_color(input.color);
}
"#;

    let without_anchor = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            static_mesh.clone(),
            ShaderPassType::Forward,
            surface_without_anchor,
            "user_surface",
        )
        .with_generated_material_source(generated_material),
    )
    .expect("auto generated material module should assemble");
    let with_anchor = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            static_mesh,
            ShaderPassType::Forward,
            surface_with_anchor,
            "user_surface",
        )
        .with_generated_material_source(generated_material),
    )
    .expect("explicit self material anchor should assemble");

    assert_eq!(with_anchor.wgsl_source, without_anchor.wgsl_source);
    assert_eq!(with_anchor.include_tokens, without_anchor.include_tokens);
    assert_eq!(
        with_anchor
            .include_tokens
            .iter()
            .filter(|token| token.as_str() == GENERATED_MATERIAL_MODULE_IMPORT_PATH)
            .count(),
        1
    );
    assert!(!with_anchor
        .wgsl_source
        .contains("#include <self::material>"));
}

#[test]
fn render_shader_template_reports_unknown_surface_module() {
    let static_mesh = static_mesh_descriptor();
    let source = r#"
#include <project::missing>

fn user_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    return zr_surface_from_base_color(input.color);
}
"#;

    let error = assemble_material_shader_template(MaterialShaderTemplateRequest::new(
        static_mesh,
        ShaderPassType::Forward,
        source,
        "user_surface",
    ))
    .expect_err("undeclared source module should fail assembly");

    assert_eq!(
        error,
        ShaderTemplateAssemblyError::UnknownModuleInclude {
            token: "project::missing".to_string(),
        }
    );
}
