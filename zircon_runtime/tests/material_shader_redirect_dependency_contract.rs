use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::{
    project::{AssetMetaDocument, AssetSourceUnit, ProjectManager, ProjectManifest, ProjectPaths},
    AssetKind, AssetReference, AssetUri, AssetUuid, ImportedAsset, MaterialAsset, ShaderAsset,
    ShaderDependencyAsset, ShaderMaterialPropertyAsset, ShaderSourceLanguage,
};
use zircon_runtime::core::framework::render::{
    MaterialPropertyKind, RenderMaterialFallbackReason, RenderMaterialValidationError,
    ShaderAssetKind,
};
use zircon_runtime::core::resource::ResourceKind;

#[test]
fn material_readiness_reports_unresolved_shader_import_redirect_dependency() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "RedirectImport"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/redirect_surface.zshader"

[overrides]
base_color = [1.0, 1.0, 1.0, 1.0]
"#,
    )
    .unwrap();
    let redirected_module = asset_reference("missing-shared", "res://shaders/missing_shared");
    let shader = shader_with_redirect_dependency(redirected_module.clone());

    let report = material.readiness_report_with_shader_contract(
        &shader,
        |reference| reference != &redirected_module,
        |_| true,
    );

    assert!(!report.is_ready());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedShaderReference { reference }
            if reference == &redirected_module
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Shader { reference }
            if reference == &redirected_module
    )));
}

#[test]
fn project_material_readiness_reports_imported_shader_redirect_dependency() {
    let root = unique_temp_project_root("material_shader_redirect_project_contract");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Shader Redirect Project",
        AssetUri::parse("res://shaders/redirect_surface").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let shader_uri = write_redirect_surface_shader_package(&paths);
    let material_uri = write_material_for_shader(&paths, &shader_uri);

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();
    let shader = load_shader(&manager, &shader_uri);
    let material = load_material(&manager, &material_uri);
    let redirected_module = shader.dependencies[0].reference.clone();

    let shader_readiness = shader.readiness_report();
    assert!(shader_readiness.has_redirected_import_dependencies());
    assert_eq!(
        shader_readiness.imports[0].source_diagnostic.as_deref(),
        Some(
            "shader import `shader_redirect_project::missing` is redirected to `res://shaders/missing_shared`"
        )
    );

    let report = material.readiness_report_with_shader_contract(
        &shader,
        |reference| {
            manager
                .registry()
                .get_by_locator(&reference.locator)
                .is_some()
        },
        |_| true,
    );

    assert!(!report.is_ready());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedShaderReference { reference }
            if reference == &redirected_module
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Shader { reference }
            if reference == &redirected_module
    )));

    let _ = fs::remove_dir_all(root);
}

fn shader_with_redirect_dependency(redirected_module: AssetReference) -> ShaderAsset {
    ShaderAsset {
        uri: AssetUri::parse("res://shaders/redirect_surface.zshader").unwrap(),
        kind: ShaderAssetKind::Surface,
        source_language: ShaderSourceLanguage::Wgsl,
        source: "fn zr_material_surface() {}".to_string(),
        wgsl_source: "fn zr_material_surface() {}".to_string(),
        import_path: Some("shader_redirect_sandbox::surface".to_string()),
        entry_points: Vec::new(),
        dependencies: vec![ShaderDependencyAsset {
            kind: ResourceKind::Shader,
            reference: redirected_module,
        }],
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: vec![ShaderMaterialPropertyAsset {
            name: "base_color".to_string(),
            kind: MaterialPropertyKind::Vec4,
            required: true,
            default: None,
            editor: Default::default(),
        }],
        options: Vec::new(),
        texture_slots: Vec::new(),
        shading_model: None,
        render_state: Default::default(),
        queue: None,
        disabled_passes: Vec::new(),
        resources: Vec::new(),
        material_property_layout: Default::default(),
        material_option_table: Default::default(),
        generated_material_wgsl: String::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn asset_reference(label: &str, uri: &str) -> AssetReference {
    AssetReference::new(
        AssetUuid::from_stable_label(label),
        AssetUri::parse(uri).unwrap(),
    )
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zircon_runtime_{label}_{}_{}",
        std::process::id(),
        timestamp
    ))
}

fn write_redirect_surface_shader_package(paths: &ProjectPaths) -> AssetUri {
    let shader_uri = AssetUri::parse("res://shaders/redirect_surface").unwrap();
    let shader_meta_path = paths
        .assets_root()
        .join("shaders")
        .join("redirect_surface.zmeta");
    let mut shader_meta =
        AssetMetaDocument::new(AssetUuid::new(), shader_uri.clone(), AssetKind::Shader);
    shader_meta.unit = AssetSourceUnit::Compound;
    fs::create_dir_all(shader_meta_path.parent().unwrap()).unwrap();
    shader_meta.save(&shader_meta_path).unwrap();

    let shader_dir = paths.assets_root().join("shaders").join("redirect_surface");
    fs::create_dir_all(&shader_dir).unwrap();
    fs::write(
        shader_dir.join("redirect_surface.zshader"),
        r#"
kind = "surface"
version = 2
shading_model = "standard_pbr"
import_path = "shader_redirect_project::surface"
wgsl_files = ["redirect_surface.wgsl"]

[[imports]]
source = "shader_redirect_project::missing"
redirect = { uuid = "22222222-2222-4222-8222-222222222222", url = "res://shaders/missing_shared" }
"#,
    )
    .unwrap();
    fs::write(
        shader_dir.join("redirect_surface.wgsl"),
        r#"
#include <shader_redirect_project::missing>

fn zr_material_surface(input: ZrSurfaceInput) -> ZrSurfaceOutput {
    return shader_redirect_project_missing_surface(input);
}
"#,
    )
    .unwrap();
    shader_uri
}

fn write_material_for_shader(paths: &ProjectPaths, shader_uri: &AssetUri) -> AssetUri {
    let material_uri = AssetUri::parse("res://materials/redirect_surface.zmaterial").unwrap();
    let material_path = paths
        .assets_root()
        .join("materials")
        .join("redirect_surface.zmaterial");
    fs::create_dir_all(material_path.parent().unwrap()).unwrap();
    fs::write(
        material_path,
        format!(
            r#"
version = 2
name = "RedirectSurface"

[shader]
uuid = "11111111-1111-4111-8111-111111111111"
url = "{shader_uri}"
"#
        ),
    )
    .unwrap();
    material_uri
}

fn load_shader(manager: &ProjectManager, shader_uri: &AssetUri) -> ShaderAsset {
    match manager.load_artifact(shader_uri).unwrap() {
        ImportedAsset::Shader(shader) => shader,
        other => panic!("unexpected shader artifact: {other:?}"),
    }
}

fn load_material(manager: &ProjectManager, material_uri: &AssetUri) -> MaterialAsset {
    match manager.load_artifact(material_uri).unwrap() {
        ImportedAsset::Material(material) => material,
        other => panic!("unexpected material artifact: {other:?}"),
    }
}
