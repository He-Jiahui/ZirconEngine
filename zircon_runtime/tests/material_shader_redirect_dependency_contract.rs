use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::{
    project::{AssetMetaDocument, AssetSourceUnit, ProjectManager, ProjectManifest, ProjectPaths},
    AssetKind, AssetReference, AssetUri, AssetUuid, ImportedAsset, MaterialAsset, ShaderAsset,
    ShaderDependencyAsset, ShaderMaterialPropertyAsset, ShaderSourceLanguage, ZMaterialDocument,
};
use zircon_runtime::core::framework::render::{
    MaterialPropertyKind, RenderMaterialFallbackReason, RenderMaterialValidationError,
    ShaderAssetKind,
};
use zircon_runtime::core::resource::ResourceKind;

#[test]
fn material_readiness_reports_unresolved_shader_import_redirect_dependency() {
    let material = material_with_shader(asset_reference(
        "redirect-surface",
        "res://shaders/redirect_surface.zshader",
    ));
    let redirected_module = asset_reference("missing-shared", "res://shaders/missing_shared");
    let shader = shader_with_redirect_dependency(redirected_module.clone());

    let report = material.readiness_report_with_shader_contract(
        &shader,
        |reference| !has_same_asset_identity(reference, &redirected_module),
        |_| true,
    );

    assert!(!report.is_ready());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedShaderReference { reference }
            if has_same_asset_identity(reference, &redirected_module)
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Shader { reference }
            if has_same_asset_identity(reference, &redirected_module)
    )));
}

#[test]
fn project_material_readiness_reports_imported_shader_redirect_dependency() {
    let root = unique_temp_project_root("material_shader_redirect_project_contract");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Shader Redirect Project",
        AssetUri::parse("res://shaders/redirect_surface").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let shader = write_redirect_surface_shader_package(&paths);
    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();
    let material_uri = write_material_for_shader(&paths, &manager, &shader);
    manager.scan_and_import().unwrap();
    let shader = load_shader(&manager, &shader.locator);
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
            if has_same_asset_identity(reference, &redirected_module)
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Shader { reference }
            if has_same_asset_identity(reference, &redirected_module)
    )));

    let _ = fs::remove_dir_all(root);
}

fn shader_with_redirect_dependency(redirected_module: AssetReference) -> ShaderAsset {
    let surface_source = "fn zr_material_surface(_input: ZrVertexOutput) -> ZrSurfaceOutput { return zr_surface_from_base_color(vec4<f32>(1.0)); }";
    ShaderAsset {
        uri: AssetUri::parse("res://shaders/redirect_surface.zshader").unwrap(),
        kind: ShaderAssetKind::Surface,
        source_language: ShaderSourceLanguage::Wgsl,
        source: surface_source.to_string(),
        wgsl_source: surface_source.to_string(),
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
        shading_model: Some("standard_pbr".to_string()),
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

fn material_with_shader(shader: AssetReference) -> MaterialAsset {
    MaterialAsset::from_zmaterial_document(material_document_with_shader(shader))
}

fn material_document_with_shader(shader: AssetReference) -> ZMaterialDocument {
    ZMaterialDocument {
        version: 2,
        name: Some("RedirectImport".to_owned()),
        shader,
        parent: None,
        options: BTreeMap::new(),
        overrides: BTreeMap::new(),
        textures: BTreeMap::new(),
        queue: None,
        editor: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn has_same_asset_identity(reference: &AssetReference, expected: &AssetReference) -> bool {
    reference.uuid == expected.uuid
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

fn write_redirect_surface_shader_package(paths: &ProjectPaths) -> AssetReference {
    let shader = asset_reference("redirect-surface", "res://shaders/redirect_surface");
    let shader_meta_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join("redirect_surface.zmeta");
    let mut shader_meta =
        AssetMetaDocument::new(shader.uuid, shader.locator.clone(), AssetKind::Shader);
    shader_meta.unit = AssetSourceUnit::Compound;
    fs::create_dir_all(shader_meta_path.parent().unwrap()).unwrap();
    shader_meta.save(&shader_meta_path).unwrap();

    let shader_dir = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join("redirect_surface");
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
    shader
}

fn write_material_for_shader(
    paths: &ProjectPaths,
    manager: &ProjectManager,
    shader: &AssetReference,
) -> AssetUri {
    let material_uri = AssetUri::parse("res://materials/redirect_surface.zmaterial").unwrap();
    let material_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("materials")
        .join("redirect_surface.zmaterial");
    fs::create_dir_all(material_path.parent().unwrap()).unwrap();
    let document = material_document_with_shader(shader.clone())
        .to_project_toml_string(|reference| manager.persist_runtime_reference(reference))
        .unwrap();
    fs::write(material_path, document).unwrap();
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
