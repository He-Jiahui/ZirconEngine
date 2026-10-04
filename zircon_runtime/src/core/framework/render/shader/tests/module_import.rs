use super::{
    derive_shader_import_path, is_builtin_shader_module_token, shader_project_namespace_from_name,
    strip_wgsl_include_directives, wgsl_include_paths, ShaderImportPathDerivationError,
    GENERATED_MATERIAL_MODULE_IMPORT_PATH,
};

#[test]
fn shader_module_imports_parse_line_directives_only() {
    let source = "// #include <ignored>\n#include <project::math>\nlet s = \"#include <ignored>\";";

    assert_eq!(
        wgsl_include_paths(source),
        vec!["project::math".to_string()]
    );
}

#[test]
fn shader_module_imports_strip_directives_without_touching_comments() {
    let source = format!(
        "// #include <ignored>\n#include <{}>\nfn surface() {{}}",
        GENERATED_MATERIAL_MODULE_IMPORT_PATH
    );

    assert_eq!(
        strip_wgsl_include_directives(&source),
        "// #include <ignored>\nfn surface() {}"
    );
}

#[test]
fn shader_module_imports_classify_builtin_tokens() {
    assert!(is_builtin_shader_module_token("zr_surface_types.wgsl"));
    assert!(is_builtin_shader_module_token("zr_shadow.wgsl"));
    assert!(!is_builtin_shader_module_token("project::shadow"));
}

#[test]
fn render_shader_import_path_derivation_uses_project_namespace_and_asset_path() {
    let derived =
        derive_shader_import_path("My Shader Project", "res://shaders/cloth/common.zshader")
            .expect("shader path should derive import path");

    assert_eq!(derived.import_path, "my_shader_project::cloth::common");
    assert!(!derived.folded_terminal_directory);
    assert_eq!(
        shader_project_namespace_from_name(" 12 My Shader Project! "),
        "_12_my_shader_project"
    );
}

#[test]
fn render_shader_import_path_derivation_folds_matching_directory_and_file_name() {
    let derived = derive_shader_import_path("MyProj", "assets/shaders/noise/noise.zshader")
        .expect("same terminal directory and file should fold");

    assert_eq!(derived.import_path, "myproj::noise");
    assert!(derived.folded_terminal_directory);
}

#[test]
fn render_shader_import_path_derivation_rejects_reserved_project_namespace() {
    let error = derive_shader_import_path("self", "shaders/cloth/common.zshader")
        .expect_err("self namespace is reserved");

    assert_eq!(
        error,
        ShaderImportPathDerivationError::ReservedNamespace {
            namespace: "self".to_string()
        }
    );
}
