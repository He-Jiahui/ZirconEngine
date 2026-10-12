use super::append_generated_material_anchor_hint;
use crate::asset::assets::ZShaderDocumentV2;
use crate::core::resource::ResourceDiagnosticSeverity;

fn surface_document() -> ZShaderDocumentV2 {
    ZShaderDocumentV2::from_toml_str(
        r#"
kind = "surface"
version = 2
shading_model = "standard_pbr"
wgsl_files = ["surface.wgsl"]
"#,
    )
    .expect("surface zshader should parse")
}

#[test]
fn zshader_import_hint_reports_missing_self_material_ide_anchor() {
    let mut diagnostics = Vec::new();

    append_generated_material_anchor_hint(
        &mut diagnostics,
        &surface_document(),
        "fn zr_material_surface() { let color = zr_mat_base_color(); }",
    );

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, ResourceDiagnosticSeverity::Info);
    assert!(diagnostics[0].message.contains("#include <self::material>"));
}

#[test]
fn zshader_import_hint_ignores_present_anchor_and_commented_symbols() {
    let document = surface_document();
    let mut diagnostics = Vec::new();

    append_generated_material_anchor_hint(
        &mut diagnostics,
        &document,
        "#include <self::material>\nfn surface() { zr_mat_base_color(); }",
    );
    append_generated_material_anchor_hint(
        &mut diagnostics,
        &document,
        "// zr_mat_base_color()\n/* zr_mat_roughness() */\nfn surface() {}",
    );

    assert!(diagnostics.is_empty());
}

#[test]
fn zshader_import_hint_ignores_commented_anchor() {
    let mut diagnostics = Vec::new();

    append_generated_material_anchor_hint(
        &mut diagnostics,
        &surface_document(),
        "/*\n#include <self::material>\n*/\nfn surface() { zr_mat_base_color(); }",
    );

    assert_eq!(diagnostics.len(), 1);
}
