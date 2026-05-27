use std::{fs, path::Path};

use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::{
    AssetId, AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor,
    AssetKind, AssetMetaDocument, AssetSourceUnit, AssetUri, AssetUuid, DataAsset, DataAssetFormat,
    FunctionAssetImporter, ImportedAsset, ImportedAssetEntry, MaterialAsset, ProjectManager,
    ProjectManifest, ProjectPaths, ShaderAsset, ShaderSourceFileAsset, ShaderSourceLanguage,
    ShaderTextureSlotAsset, ZShaderDocument,
};
use crate::core::framework::render::{
    RenderShaderBindGroupLayoutDescriptor, RenderShaderBindingDescriptor,
    RenderShaderBindingResourceType, RenderShaderDefinitionValue,
    RenderShaderPipelineLayoutDescriptor, RenderShaderStage,
};
use crate::core::resource::ResourceState;
use crate::plugin::PluginPackageManifest;

#[test]
fn project_manager_writes_zmeta_schema_and_ignores_old_meta_toml_sidecars() {
    let root = unique_temp_project_root("project_manager_zmeta_schema");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "ZMetaSandbox",
        AssetUri::parse("res://data/settings.json").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let data_path = paths.assets_root().join("data").join("settings.json");
    fs::create_dir_all(data_path.parent().unwrap()).unwrap();
    fs::write(&data_path, r#"{ "answer": 42 }"#).unwrap();
    fs::write(
        paths
            .assets_root()
            .join("data")
            .join("settings.json.meta.toml"),
        "legacy sidecar must stay ignored",
    )
    .unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();

    let uri = AssetUri::parse("res://data/settings.json").unwrap();
    let record = manager.registry().get_by_locator(&uri).unwrap();
    let meta_path = paths.assets_root().join("data").join("settings.json.zmeta");
    let meta = AssetMetaDocument::load(&meta_path).unwrap();

    assert!(meta_path.exists());
    assert_eq!(meta.format_version, 6);
    assert_eq!(meta.url, uri);
    assert_eq!(meta.asset_kind, AssetKind::Data);
    assert_eq!(meta.unit, AssetSourceUnit::Single);
    assert!(meta.included_files.is_empty());
    assert_eq!(meta.entries.len(), 1);
    assert_eq!(meta.entries[0].uuid, meta.uuid);
    assert_eq!(meta.entries[0].url, uri);
    assert_eq!(meta.entries[0].asset_kind, AssetKind::Data);
    assert_eq!(record.id(), AssetId::from_asset_uuid(meta.uuid));
    assert_eq!(record.state, ResourceState::Ready);
    assert!(manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://data/settings.json.meta.toml").unwrap())
        .is_none());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_resolves_asset_references_by_uuid_before_stale_url() {
    let root = unique_temp_project_root("project_manager_reference_uuid_first");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "ReferenceSandbox",
        AssetUri::parse("res://data/renamed.json").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let data_path = paths.assets_root().join("data").join("renamed.json");
    fs::create_dir_all(data_path.parent().unwrap()).unwrap();
    fs::write(&data_path, r#"{ "renamed": true }"#).unwrap();
    let uuid = AssetUuid::new();
    let stale_url = AssetUri::parse("res://data/original.json").unwrap();
    AssetMetaDocument::new(uuid, stale_url.clone(), AssetKind::Data)
        .save(paths.assets_root().join("data").join("renamed.json.zmeta"))
        .unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();

    let current_url = AssetUri::parse("res://data/renamed.json").unwrap();
    let current_id = manager.asset_id_for_uri(&current_url).unwrap();

    assert_eq!(manager.asset_id_for_uuid(uuid), Some(current_id));
    assert_eq!(
        manager.asset_id_for_reference(uuid, &stale_url),
        Some(current_id)
    );
    assert_eq!(
        manager.stale_url_for_reference(uuid, &stale_url),
        Some(&current_url)
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_restore_refreshes_zmeta_entry_urls_after_source_rename() {
    let root = unique_temp_project_root("project_manager_rename_restore_zmeta_urls");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "RenameRestoreSandbox",
        AssetUri::parse("res://bundles/renamed.multi").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let original_source = paths.assets_root().join("bundles").join("atlas.multi");
    let renamed_source = paths.assets_root().join("bundles").join("renamed.multi");
    fs::create_dir_all(original_source.parent().unwrap()).unwrap();
    fs::write(&original_source, "atlas").unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(multi_asset_importer("multi"))
        .unwrap();
    manager.scan_and_import().unwrap();

    let original_root_uri = AssetUri::parse("res://bundles/atlas.multi").unwrap();
    let original_texture_uri = AssetUri::parse("res://bundles/atlas.multi#Texture0").unwrap();
    let original_meta_path = paths
        .assets_root()
        .join("bundles")
        .join("atlas.multi.zmeta");
    let renamed_meta_path = paths
        .assets_root()
        .join("bundles")
        .join("renamed.multi.zmeta");
    let original_meta = AssetMetaDocument::load(&original_meta_path).unwrap();
    let original_texture_uuid = original_meta
        .entries
        .iter()
        .find(|entry| entry.url == original_texture_uri)
        .expect("original texture entry")
        .uuid;
    let original_texture_id = manager
        .registry()
        .get_by_locator(&original_texture_uri)
        .expect("original texture record")
        .id();

    fs::rename(&original_source, &renamed_source).unwrap();
    fs::rename(&original_meta_path, &renamed_meta_path).unwrap();

    let mut restarted = ProjectManager::open(&root).unwrap();
    restarted.scan_and_import().unwrap();

    let renamed_root_uri = AssetUri::parse("res://bundles/renamed.multi").unwrap();
    let renamed_texture_uri = AssetUri::parse("res://bundles/renamed.multi#Texture0").unwrap();
    let restored_meta = AssetMetaDocument::load(&renamed_meta_path).unwrap();
    let restored_root = restarted
        .registry()
        .get_by_locator(&renamed_root_uri)
        .expect("restored root record should use renamed URL");
    let restored_texture = restarted
        .registry()
        .get_by_locator(&renamed_texture_uri)
        .expect("restored texture record should use renamed URL");

    assert!(restarted
        .registry()
        .get_by_locator(&original_root_uri)
        .is_none());
    assert!(restarted
        .registry()
        .get_by_locator(&original_texture_uri)
        .is_none());
    assert_eq!(restored_meta.url, renamed_root_uri);
    assert!(restored_meta
        .entries
        .iter()
        .any(|entry| entry.uuid == restored_meta.uuid && entry.url == renamed_root_uri));
    assert!(restored_meta
        .entries
        .iter()
        .any(|entry| entry.uuid == original_texture_uuid && entry.url == renamed_texture_uri));
    assert_eq!(restored_texture.id(), original_texture_id);
    assert_eq!(
        restored_root.id(),
        AssetId::from_asset_uuid(restored_meta.uuid)
    );
    assert_eq!(
        restarted.asset_id_for_reference(original_texture_uuid, &original_texture_uri),
        Some(restored_texture.id())
    );
    assert_eq!(
        restarted.stale_url_for_reference(original_texture_uuid, &original_texture_uri),
        Some(&renamed_texture_uri)
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_preserves_zmeta_subasset_uuids_across_failed_reimport() {
    let root = unique_temp_project_root("project_manager_failed_reimport_zmeta_uuid");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "FailedReimportSandbox",
        AssetUri::parse("res://bundles/atlas.flaky").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let source_path = paths.assets_root().join("bundles").join("atlas.flaky");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "atlas").unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(flaky_multi_asset_importer())
        .unwrap();
    manager.scan_and_import().unwrap();

    let root_uri = AssetUri::parse("res://bundles/atlas.flaky").unwrap();
    let texture_uri = AssetUri::parse("res://bundles/atlas.flaky#Texture0").unwrap();
    let meta_path = paths
        .assets_root()
        .join("bundles")
        .join("atlas.flaky.zmeta");
    let ready_meta = AssetMetaDocument::load(&meta_path).unwrap();
    let ready_texture_uuid = ready_meta
        .entries
        .iter()
        .find(|entry| entry.url == texture_uri)
        .expect("ready texture entry")
        .uuid;
    let ready_texture_id = manager
        .registry()
        .get_by_locator(&texture_uri)
        .expect("ready texture record")
        .id();

    fs::write(&source_path, "fail").unwrap();
    manager.scan_and_import().unwrap();

    let failed_meta = AssetMetaDocument::load(&meta_path).unwrap();
    assert_eq!(
        failed_meta.preview_state,
        crate::asset::project::PreviewState::Error
    );
    assert!(manager.registry().get_by_locator(&texture_uri).is_none());
    assert!(failed_meta
        .entries
        .iter()
        .any(|entry| entry.uuid == ready_texture_uuid && entry.url == texture_uri));

    fs::write(&source_path, "atlas-fixed").unwrap();
    manager.scan_and_import().unwrap();

    let recovered_meta = AssetMetaDocument::load(&meta_path).unwrap();
    let recovered_texture = manager
        .registry()
        .get_by_locator(&texture_uri)
        .expect("recovered texture record");
    assert_eq!(
        recovered_meta.preview_state,
        crate::asset::project::PreviewState::Ready
    );
    assert_eq!(
        recovered_meta
            .entries
            .iter()
            .find(|entry| entry.url == texture_uri)
            .expect("recovered texture entry")
            .uuid,
        ready_texture_uuid
    );
    assert_eq!(recovered_texture.id(), ready_texture_id);
    assert_eq!(
        manager.asset_id_for_reference(ready_texture_uuid, &root_uri),
        Some(ready_texture_id)
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_scans_package_asset_roots_as_package_uris() {
    let root = unique_temp_project_root("project_manager_package_zmeta");
    let package_root = unique_temp_project_root("navigation_package_zmeta");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "PackageSandbox",
        AssetUri::parse("res://data/project.json").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let project_path = paths.assets_root().join("data").join("project.json");
    fs::create_dir_all(project_path.parent().unwrap()).unwrap();
    fs::write(&project_path, r#"{ "project": true }"#).unwrap();

    let package_asset_path = package_root.join("assets").join("nav").join("agent.json");
    fs::create_dir_all(package_asset_path.parent().unwrap()).unwrap();
    fs::write(&package_asset_path, r#"{ "agent": true }"#).unwrap();

    let package_manifest = PluginPackageManifest::new("navigation", "Navigation")
        .with_package_identity("com", "zircon", "navigation");
    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_package_manifest_asset_roots(&package_manifest, &package_root)
        .unwrap();
    manager.scan_and_import().unwrap();

    let package_uri = AssetUri::parse("package://com.zircon.navigation/nav/agent.json").unwrap();
    let package_record = manager
        .registry()
        .get_by_locator(&package_uri)
        .expect("package asset record");
    let package_meta_path = package_root
        .join("assets")
        .join("nav")
        .join("agent.json.zmeta");
    let package_meta = AssetMetaDocument::load(&package_meta_path).unwrap();

    assert_eq!(package_manifest.package_id(), "com.zircon.navigation");
    assert_eq!(
        package_manifest.asset_roots_or_default(),
        vec!["assets".to_string()]
    );
    assert_eq!(
        manager.source_path_for_uri(&package_uri).unwrap(),
        package_asset_path
    );
    assert_eq!(package_meta.url, package_uri);
    assert_eq!(package_meta.asset_kind, AssetKind::Data);
    assert_eq!(
        package_record.id(),
        AssetId::from_asset_uuid(package_meta.uuid)
    );
    assert!(manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://data/project.json").unwrap())
        .is_some());

    let error = manager
        .source_path_for_uri(
            &AssetUri::parse("package://com.zircon.missing/nav/agent.json").unwrap(),
        )
        .expect_err("unknown package should be rejected");
    assert!(error
        .to_string()
        .contains("unknown package com.zircon.missing"));

    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(package_root);
}

#[test]
fn project_manager_imports_compound_zshader_package_with_subassets() {
    let root = unique_temp_project_root("project_manager_compound_zshader");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "ShaderPackageSandbox",
        AssetUri::parse("res://shaders/unlit_shader").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let shader_uri = AssetUri::parse("res://shaders/unlit_shader").unwrap();
    let shader_meta_path = paths
        .assets_root()
        .join("shaders")
        .join("unlit_shader.zmeta");
    let mut shader_meta =
        AssetMetaDocument::new(AssetUuid::new(), shader_uri.clone(), AssetKind::Shader);
    shader_meta.unit = AssetSourceUnit::Compound;
    shader_meta.save(&shader_meta_path).unwrap();

    let shader_dir = paths.assets_root().join("shaders").join("unlit_shader");
    fs::create_dir_all(&shader_dir).unwrap();
    fs::write(
        shader_dir.join("unlit.zshader"),
        r#"
version = 1
import_path = "zircon::unlit"
wgsl_files = ["unlit.wgsl"]
shader_defs = ["USE_UNLIT", "ALPHA_CLIP"]

[[shader_def_values]]
name = "TONEMAPPING_LUT_TEXTURE_BINDING_INDEX"
kind = "uint"
value = 2

[[shader_def_values]]
name = "ENABLE_FOG"
kind = "bool"
value = false

[[shader_def_values]]
name = "DEBUG_MODE"
kind = "int"
value = -1

[pipeline_layout]
push_constant_ranges = ["draw_index:0..4"]

[[pipeline_layout.bind_groups]]
group = 3
label = "material"

[[pipeline_layout.bind_groups.bindings]]
binding = 0
label = "material_uniforms"
resource_type = "uniform_buffer"
visibility = ["vertex", "fragment"]

[[imports]]
source = "zircon::lighting"
redirect = { uuid = "22222222-2222-4222-8222-222222222222", url = "res://shaders/shared_lighting" }

[[imports]]
source = "naga_oil::math"

[[entry_points]]
name = "vs_main"
stage = "vertex"
file = "unlit.wgsl"

[[entry_points]]
name = "fs_main"
stage = "fragment"
file = "unlit.wgsl"

[[properties]]
name = "base_color"
kind = "vec4"
required = true
default = [1.0, 1.0, 1.0, 1.0]
editor = { label = "Base Color", group = "Surface" }

[[texture_slots]]
name = "base_color"
kind = "texture2d"
default = "white"
sampler = "linear_repeat"
group = "Surface"
label = "Base Color Texture"
"#,
    )
    .unwrap();
    fs::write(
        shader_dir.join("unlit.wgsl"),
        r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    let x = f32(i32(vertex_index) - 1);
    return vec4f(x, 0.0, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    let base_color = vec4f(1.0, 1.0, 1.0, 1.0);
    return base_color;
}
"#,
    )
    .unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();

    let imported_meta = AssetMetaDocument::load(&shader_meta_path).unwrap();
    let shader_record = manager
        .registry()
        .get_by_locator(&shader_uri)
        .expect("compound shader root record");
    let zshader_subasset = AssetUri::parse("res://shaders/unlit_shader#zshader:unlit.zshader")
        .expect("zshader subasset uri");
    let wgsl_subasset =
        AssetUri::parse("res://shaders/unlit_shader#wgsl:unlit.wgsl").expect("wgsl subasset uri");

    assert_eq!(imported_meta.unit, AssetSourceUnit::Compound);
    assert_eq!(imported_meta.asset_kind, AssetKind::Shader);
    assert!(imported_meta
        .included_files
        .contains(&AssetUri::parse("res://shaders/unlit_shader/unlit.zshader").unwrap()));
    assert!(imported_meta
        .included_files
        .contains(&AssetUri::parse("res://shaders/unlit_shader/unlit.wgsl").unwrap()));
    assert!(imported_meta
        .entries
        .iter()
        .any(|entry| entry.url == shader_uri));
    assert!(imported_meta
        .entries
        .iter()
        .any(|entry| entry.url == zshader_subasset && entry.asset_kind == AssetKind::Data));
    assert!(imported_meta
        .entries
        .iter()
        .any(|entry| entry.url == wgsl_subasset && entry.asset_kind == AssetKind::Data));
    assert_eq!(
        shader_record.id(),
        AssetId::from_asset_uuid(imported_meta.uuid)
    );

    match manager.load_artifact(&shader_uri).unwrap() {
        ImportedAsset::Shader(shader) => {
            assert_eq!(shader.source_files.len(), 1);
            assert_eq!(shader.source_files[0].path, "unlit.wgsl");
            assert_eq!(shader.import_path.as_deref(), Some("zircon::unlit"));
            assert_eq!(shader.imports.len(), 2);
            assert_eq!(shader.imports[0].source, "zircon::lighting");
            assert_eq!(
                shader.imports[0]
                    .redirect
                    .as_ref()
                    .expect("redirected shader import")
                    .locator,
                AssetUri::parse("res://shaders/shared_lighting").unwrap()
            );
            assert_eq!(shader.imports[1].source, "naga_oil::math");
            assert!(shader.imports[1].redirect.is_none());
            assert_eq!(shader.dependencies.len(), 1);
            assert_eq!(
                shader.dependencies()[0].reference.locator,
                AssetUri::parse("res://shaders/shared_lighting").unwrap()
            );
            assert_eq!(shader.entry_points.len(), 2);
            assert_eq!(
                shader.shader_defs,
                vec![
                    RenderShaderDefinitionValue::from("USE_UNLIT"),
                    RenderShaderDefinitionValue::from("ALPHA_CLIP"),
                    RenderShaderDefinitionValue::uint("TONEMAPPING_LUT_TEXTURE_BINDING_INDEX", 2),
                    RenderShaderDefinitionValue::bool("ENABLE_FOG", false),
                    RenderShaderDefinitionValue::int("DEBUG_MODE", -1),
                ]
            );
            assert_eq!(shader.variant_keys()[0].defines, shader.shader_defs);
            assert_eq!(shader.property_schema.len(), 1);
            assert_eq!(shader.property_schema[0].name, "base_color");
            assert_eq!(shader.texture_slots.len(), 1);
            assert_eq!(shader.texture_slots[0].name, "base_color");
            assert_eq!(shader.texture_slots[0].default.as_deref(), Some("white"));
            assert_eq!(
                shader.pipeline_layout,
                RenderShaderPipelineLayoutDescriptor {
                    bind_groups: vec![RenderShaderBindGroupLayoutDescriptor {
                        group: 3,
                        label: Some("material".to_string()),
                        bindings: vec![RenderShaderBindingDescriptor {
                            binding: 0,
                            label: Some("material_uniforms".to_string()),
                            resource_type: RenderShaderBindingResourceType::UniformBuffer,
                            visibility: vec![
                                RenderShaderStage::Vertex,
                                RenderShaderStage::Fragment,
                            ],
                        }],
                    }],
                    push_constant_ranges: vec!["draw_index:0..4".to_string()],
                }
            );
            assert!(shader.validation_diagnostics.is_empty());

            let readiness = shader.readiness_report();
            assert!(readiness.is_ready());
            assert!(readiness.uses_runtime_wgsl());
            assert!(readiness.has_pipeline_layout);
            assert!(readiness.has_redirected_import_dependencies());
            assert_eq!(readiness.dependency_count, 1);
            assert_eq!(readiness.imports.len(), 2);
            assert_eq!(readiness.imports[0].source, "zircon::lighting");
            assert!(readiness.imports[0].contributes_dependency);
            assert_eq!(readiness.imports[1].source, "naga_oil::math");
            assert!(!readiness.imports[1].contributes_dependency);
            assert_eq!(readiness.entry_points.len(), 2);
            assert!(readiness
                .entry_points
                .iter()
                .all(|entry| entry.diagnostic.is_none()));
            assert_eq!(readiness.shader_defs.len(), 5);
            assert!(readiness
                .shader_defs
                .iter()
                .all(|definition| definition.diagnostic.is_none()));
            assert_eq!(readiness.shader_defs[2].value.value_as_string(), "2");
            assert_eq!(readiness.shader_defs[3].value.value_as_string(), "false");
            assert_eq!(readiness.shader_defs[4].value.value_as_string(), "-1");
            assert!(readiness.validation_diagnostics.is_empty());

            let mut material = material_for_shader(&shader_uri);
            material.property_values.insert(
                "base_color".to_string(),
                toml::Value::Array(vec![
                    toml::Value::Float(1.0),
                    toml::Value::Float(0.8),
                    toml::Value::Float(0.2),
                    toml::Value::Float(1.0),
                ]),
            );
            assert!(material.shader_property_diagnostics(&shader).is_empty());
            material
                .property_values
                .insert("unknown".to_string(), toml::Value::Boolean(true));
            assert!(material
                .shader_property_diagnostics(&shader)
                .iter()
                .any(|diagnostic| diagnostic.contains("not declared")));
        }
        other => panic!("unexpected compound shader artifact: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_imports_zshader_with_wgsl_capture_diagnostics() {
    let root = unique_temp_project_root("project_manager_zshader_capture_diagnostics");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "ShaderCaptureSandbox",
        AssetUri::parse("res://shaders/capture_shader").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let shader_uri = AssetUri::parse("res://shaders/capture_shader").unwrap();
    let shader_meta_path = paths
        .assets_root()
        .join("shaders")
        .join("capture_shader.zmeta");
    let mut shader_meta =
        AssetMetaDocument::new(AssetUuid::new(), shader_uri.clone(), AssetKind::Shader);
    shader_meta.unit = AssetSourceUnit::Compound;
    shader_meta.save(&shader_meta_path).unwrap();

    let shader_dir = paths.assets_root().join("shaders").join("capture_shader");
    fs::create_dir_all(&shader_dir).unwrap();
    fs::write(
        shader_dir.join("capture.zshader"),
        r#"
version = 1
wgsl_files = ["capture.wgsl"]

[[entry_points]]
name = "vs_main"
stage = "vertex"

[[properties]]
name = "base_color"
kind = "vec4"

[[texture_slots]]
name = "albedo"
kind = "texture2d"
"#,
    )
    .unwrap();
    fs::write(
        shader_dir.join("capture.wgsl"),
        r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    return vec4f(f32(vertex_index), 0.0, 0.0, 1.0);
}
"#,
    )
    .unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();

    match manager.load_artifact(&shader_uri).unwrap() {
        ImportedAsset::Shader(shader) => {
            assert!(shader
                .validation_diagnostics
                .iter()
                .any(|diagnostic| diagnostic
                    .contains("wgsl_capture property `base_color` was not found")));
            assert!(shader
                .validation_diagnostics
                .iter()
                .any(|diagnostic| diagnostic
                    .contains("wgsl_capture texture slot `albedo` was not found")));
            let readiness = shader.readiness_report();
            assert!(!readiness.is_ready());
            assert!(readiness.uses_runtime_wgsl());
            assert_eq!(
                readiness.validation_diagnostics,
                shader.validation_diagnostics
            );
            assert!(readiness
                .validation_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("wgsl_capture property `base_color`")));
            assert!(readiness
                .validation_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("wgsl_capture texture slot `albedo`")));
        }
        other => panic!("unexpected compound shader artifact: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn zshader_typed_shader_definition_rows_validate_kind_and_value() {
    let document = ZShaderDocument::from_toml_str(
        r#"
version = 1
shader_defs = ["LEGACY_FLAG"]

[[shader_def_values]]
name = "ENABLE_FOG"
kind = "bool"
value = false

[[shader_def_values]]
name = "BINDING_INDEX"
kind = "uint"
value = 4

[[shader_def_values]]
name = "DEBUG_MODE"
kind = "int"
value = -1
"#,
    )
    .unwrap();

    assert_eq!(
        document.shader_definition_values().unwrap(),
        vec![
            RenderShaderDefinitionValue::from("LEGACY_FLAG"),
            RenderShaderDefinitionValue::bool("ENABLE_FOG", false),
            RenderShaderDefinitionValue::uint("BINDING_INDEX", 4),
            RenderShaderDefinitionValue::int("DEBUG_MODE", -1),
        ]
    );

    let unknown_kind = ZShaderDocument::from_toml_str(
        r#"
[[shader_def_values]]
name = "BAD_KIND"
kind = "float"
value = 1.0
"#,
    )
    .unwrap();
    assert!(unknown_kind
        .shader_definition_values()
        .unwrap_err()
        .contains("unsupported kind `float`"));

    let non_bool = ZShaderDocument::from_toml_str(
        r#"
[[shader_def_values]]
name = "ENABLE_FOG"
kind = "bool"
value = 1
"#,
    )
    .unwrap();
    assert!(non_bool
        .shader_definition_values()
        .unwrap_err()
        .contains("value is not a boolean"));

    let negative_uint = ZShaderDocument::from_toml_str(
        r#"
[[shader_def_values]]
name = "BINDING_INDEX"
kind = "uint"
value = -1
"#,
    )
    .unwrap();
    assert!(negative_uint
        .shader_definition_values()
        .unwrap_err()
        .contains("value is not a u32 integer"));
}

#[test]
fn documented_zmeta_shader_material_fixture_parses() {
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("runtime crate should have repo parent")
        .join("docs")
        .join("assets-and-rendering")
        .join("fixtures")
        .join("zmeta-shader-material")
        .join("assets");

    let shader_meta =
        AssetMetaDocument::load(fixture_root.join("shaders").join("unlit_shader.zmeta")).unwrap();
    let shader_uri = AssetUri::parse("res://shaders/unlit_shader").unwrap();
    let zshader_uri = AssetUri::parse("res://shaders/unlit_shader#zshader:unlit.zshader").unwrap();
    let wgsl_uri = AssetUri::parse("res://shaders/unlit_shader#wgsl:unlit.wgsl").unwrap();

    assert_eq!(shader_meta.url, shader_uri);
    assert_eq!(shader_meta.asset_kind, AssetKind::Shader);
    assert_eq!(shader_meta.unit, AssetSourceUnit::Compound);
    assert_eq!(shader_meta.entries.len(), 3);
    assert!(shader_meta
        .entries
        .iter()
        .any(|entry| entry.url == zshader_uri && entry.asset_kind == AssetKind::Data));
    assert!(shader_meta
        .entries
        .iter()
        .any(|entry| entry.url == wgsl_uri && entry.asset_kind == AssetKind::Data));

    let zshader = ZShaderDocument::from_toml_str(
        &fs::read_to_string(
            fixture_root
                .join("shaders")
                .join("unlit_shader")
                .join("unlit.zshader"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(zshader.wgsl_files, vec!["unlit.wgsl"]);
    assert_eq!(zshader.import_path.as_deref(), Some("zircon::unlit"));
    assert_eq!(zshader.shader_defs, vec!["USE_UNLIT"]);
    assert_eq!(
        zshader.shader_definition_values().unwrap(),
        vec![RenderShaderDefinitionValue::from("USE_UNLIT")]
    );
    assert_eq!(zshader.entry_points.len(), 2);
    assert_eq!(zshader.properties[0].name, "base_color");
    let fixture_wgsl = fs::read_to_string(
        fixture_root
            .join("shaders")
            .join("unlit_shader")
            .join("unlit.wgsl"),
    )
    .unwrap();
    let shader_asset = ShaderAsset {
        uri: shader_uri.clone(),
        source_language: ShaderSourceLanguage::Wgsl,
        source: fixture_wgsl.clone(),
        wgsl_source: fixture_wgsl,
        import_path: zshader.import_path.clone(),
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: vec![ShaderSourceFileAsset {
            path: "unlit.wgsl".to_string(),
            url: AssetUri::parse("res://shaders/unlit_shader/unlit.wgsl").unwrap(),
        }],
        imports: Vec::new(),
        shader_defs: zshader.shader_definition_values().unwrap(),
        property_schema: zshader.properties.clone(),
        texture_slots: zshader
            .texture_slots
            .iter()
            .map(ShaderTextureSlotAsset::from)
            .collect(),
        editor: zshader.editor.clone(),
        pipeline_layout: zshader.pipeline_layout.clone(),
        validation_diagnostics: Vec::new(),
    };
    assert!(
        crate::asset::validate_wgsl_captures(&shader_asset).is_empty(),
        "documented fixture WGSL should reference every declared shader property and texture slot"
    );

    let material = MaterialAsset::from_toml_str(
        &fs::read_to_string(fixture_root.join("materials").join("hero_unlit.zmaterial")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        material.shader.uuid,
        "11111111-2222-4333-8444-555555555555".parse().unwrap()
    );
    assert_eq!(material.shader.locator, shader_uri);
    assert!(material.property_values.contains_key("base_color"));
    assert!(material.texture_slots.contains_key("base_color"));

    let material_meta = AssetMetaDocument::load(
        fixture_root
            .join("materials")
            .join("hero_unlit.zmaterial.zmeta"),
    )
    .unwrap();
    assert_eq!(material_meta.asset_kind, AssetKind::Material);
    assert_eq!(material_meta.dependencies.len(), 2);
    assert!(material_meta.dependencies.contains(&shader_uri));
    assert!(material_meta
        .dependencies
        .contains(&AssetUri::parse("res://textures/hero_albedo.png").unwrap()));
}

fn multi_asset_importer(extension: &'static str) -> FunctionAssetImporter {
    FunctionAssetImporter::new(
        AssetImporterDescriptor::new("test.multi.bundle", "test.multi", AssetKind::Data, 1)
            .with_source_extensions([extension])
            .with_additional_output_kinds([AssetKind::Texture]),
        import_multi_asset_bundle,
    )
}

fn material_for_shader(shader_uri: &AssetUri) -> MaterialAsset {
    MaterialAsset {
        name: Some("UnlitMaterial".to_string()),
        shader: crate::asset::AssetReference::from_locator(shader_uri.clone()),
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: crate::asset::AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn flaky_multi_asset_importer() -> FunctionAssetImporter {
    FunctionAssetImporter::new(
        AssetImporterDescriptor::new("test.multi.flaky", "test.multi", AssetKind::Data, 1)
            .with_source_extensions(["flaky"])
            .with_additional_output_kinds([AssetKind::Texture]),
        import_flaky_multi_asset_bundle,
    )
}

fn import_multi_asset_bundle(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    import_multi_asset_bundle_with_text(context, context.source_text()?)
}

fn import_flaky_multi_asset_bundle(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    if text == "fail" {
        return Err(AssetImportError::Parse(
            "transient flaky import failure".to_string(),
        ));
    }
    import_multi_asset_bundle_with_text(context, text)
}

fn import_multi_asset_bundle_with_text(
    context: &AssetImportContext,
    text: String,
) -> Result<AssetImportOutcome, AssetImportError> {
    let texture_uri = AssetUri::parse(&format!("{}#Texture0", context.uri)).unwrap();
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format: DataAssetFormat::Json,
            text,
            canonical_json: serde_json::json!({ "bundle": true }),
        }),
    )
    .with_entry(ImportedAssetEntry::new(
        texture_uri.clone(),
        ImportedAsset::Texture(crate::asset::TextureAsset::new_rgba8(
            texture_uri,
            1,
            1,
            vec![255, 0, 255, 255],
        )),
    )))
}
