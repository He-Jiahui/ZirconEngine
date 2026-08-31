use crate::asset::assets::ZShaderDocumentV2;
use crate::core::framework::render::{GENERATED_MATERIAL_MODULE_IMPORT_PATH, wgsl_include_paths};
use crate::core::resource::{ResourceDiagnostic, ResourceDiagnosticSeverity};

pub(super) fn append_generated_material_anchor_hint(
    diagnostics: &mut Vec<ResourceDiagnostic>,
    document: &ZShaderDocumentV2,
    wgsl_source: &str,
) {
    if !document.kind().participates_in_material_variants()
        || !wgsl_needs_generated_material_anchor_hint(wgsl_source)
    {
        return;
    }

    diagnostics.push(ResourceDiagnostic {
        severity: ResourceDiagnosticSeverity::Info,
        message: "surface WGSL uses generated `zr_mat_*` symbols without `#include <self::material>`; add the include as an IDE navigation anchor (runtime assembly is unchanged)".to_string(),
    });
}

fn wgsl_needs_generated_material_anchor_hint(wgsl_source: &str) -> bool {
    if !wgsl_source.contains("zr_mat_") {
        return false;
    }

    let authored_source = wgsl_source_without_comments(wgsl_source);
    if wgsl_include_paths(&authored_source)
        .iter()
        .any(|path| path == GENERATED_MATERIAL_MODULE_IMPORT_PATH)
        || !wgsl_uses_generated_material_symbol(&authored_source)
    {
        return false;
    }
    true
}

fn wgsl_source_without_comments(source: &str) -> String {
    // WGSL block comments can nest. Preserve newlines and token boundaries for line directives.
    let bytes = source.as_bytes();
    let mut authored = Vec::with_capacity(bytes.len());
    let mut index = 0;
    let mut block_comment_depth = 0_u32;

    while index < bytes.len() {
        if block_comment_depth > 0 {
            if bytes.get(index..index + 2) == Some(b"/*") {
                block_comment_depth += 1;
                index += 2;
            } else if bytes.get(index..index + 2) == Some(b"*/") {
                block_comment_depth -= 1;
                index += 2;
            } else {
                if bytes[index] == b'\n' {
                    authored.push(b'\n');
                }
                index += 1;
            }
            continue;
        }

        if bytes.get(index..index + 2) == Some(b"//") {
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"/*") {
            block_comment_depth = 1;
            authored.push(b' ');
            index += 2;
            continue;
        }
        authored.push(bytes[index]);
        index += 1;
    }

    String::from_utf8(authored).expect("removing WGSL comments preserves UTF-8")
}

fn wgsl_uses_generated_material_symbol(source: &str) -> bool {
    source
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .any(|identifier| identifier.starts_with("zr_mat_"))
}

#[cfg(test)]
#[path = "generated_material_anchor_hint/symbol_prefilter_tests.rs"]
mod symbol_prefilter_tests;

#[cfg(test)]
mod tests {
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
}
