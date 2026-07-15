use super::*;

#[test]
fn project_manager_imports_compound_zshader_package_with_subassets() {
    let root = unique_temp_project_root("project_manager_compound_zshader");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "ShaderPackageSandbox",
        AssetUri::parse("res://shaders/unlit_shader").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let shader_uri = AssetUri::parse("res://shaders/unlit_shader").unwrap();
    let shader_meta_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join("unlit_shader.zmeta");
    let mut shader_meta =
        AssetMetaDocument::new(AssetUuid::new(), shader_uri.clone(), AssetKind::Shader);
    shader_meta.unit = AssetSourceUnit::Compound;
    shader_meta.save(&shader_meta_path).unwrap();

    let shader_dir = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join("unlit_shader");
    fs::create_dir_all(&shader_dir).unwrap();
    fs::write(
        shader_dir.join("unlit.zshader"),
        r#"
kind = "surface"
version = 2
shading_model = "unlit"
import_path = "zircon::unlit"
wgsl_files = ["unlit.wgsl"]

[[options]]
name = "USE_UNLIT"
kind = "bool"
default = true

[[options]]
name = "ALPHA_CLIP"
kind = "bool"
default = false

[[imports]]
source = "zircon::lighting"
redirect = { uuid = "22222222-2222-4222-8222-222222222222", url = "res://shaders/shared_lighting" }

[[imports]]
source = "naga_oil::math"

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
#include <zircon::lighting>
#include <naga_oil::math>

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
            assert_eq!(shader.kind, ShaderAssetKind::Surface);
            assert_eq!(shader.source_files.len(), 1);
            assert_eq!(shader.source_files[0].path, "unlit.wgsl");
            assert_eq!(shader.import_path.as_deref(), Some("zircon::unlit"));
            assert_eq!(shader.shading_model.as_deref(), Some("unlit"));
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
            assert!(shader.shader_defs.is_empty());
            assert_eq!(shader.variant_keys()[0].defines, shader.shader_defs);
            assert_eq!(shader.options.len(), 2);
            assert_eq!(shader.options[0].name, "USE_UNLIT");
            assert_eq!(shader.options[0].default, Some(toml::Value::Boolean(true)));
            assert_eq!(shader.options[1].name, "ALPHA_CLIP");
            assert_eq!(shader.options[1].default, Some(toml::Value::Boolean(false)));
            assert_eq!(shader.property_schema.len(), 1);
            assert_eq!(shader.property_schema[0].name, "base_color");
            assert_eq!(shader.texture_slots.len(), 1);
            assert_eq!(shader.texture_slots[0].name, "base_color");
            assert_eq!(shader.texture_slots[0].default.as_deref(), Some("white"));
            assert_eq!(shader.pipeline_layout, Default::default());
            assert!(shader.validation_diagnostics.is_empty());

            let readiness = shader.readiness_report();
            assert!(readiness.is_ready());
            assert!(readiness.uses_runtime_wgsl());
            assert!(!readiness.has_pipeline_layout);
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
            assert!(readiness.shader_defs.is_empty());
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
fn compound_shader_persisted_reference_uses_zmeta_source_without_changing_uuid() {
    let root = unique_temp_project_root("compound_shader_persisted_reference");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Persisted Reference Sandbox",
        AssetUri::parse("res://shaders/redirect_surface").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let shader_uri = write_include_shader_package(&paths, "redirect_surface", "redirect::surface");
    let meta_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders/redirect_surface.zmeta");
    let uuid = AssetMetaDocument::load(&meta_path).unwrap().uuid;

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();

    let persisted = manager
        .persist_runtime_reference(&crate::asset::AssetReference::new(uuid, shader_uri))
        .unwrap();
    let project = persisted
        .project_ref()
        .expect("project persisted reference");

    assert_eq!(project.guid(), uuid);
    assert_eq!(
        project.path_hint().as_str(),
        "assets/shaders/redirect_surface.zmeta"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_derives_include_shader_import_path_from_project_and_package_path() {
    let root = unique_temp_project_root("project_manager_shader_import_path_derivation");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Shader Package Sandbox",
        AssetUri::parse("res://shaders/noise").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let shader_uri = AssetUri::parse("res://shaders/noise").unwrap();
    let shader_meta_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join("noise.zmeta");
    let mut shader_meta =
        AssetMetaDocument::new(AssetUuid::new(), shader_uri.clone(), AssetKind::Shader);
    shader_meta.unit = AssetSourceUnit::Compound;
    fs::create_dir_all(shader_meta_path.parent().unwrap()).unwrap();
    shader_meta.save(&shader_meta_path).unwrap();

    let shader_dir = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join("noise");
    fs::create_dir_all(&shader_dir).unwrap();
    fs::write(
        shader_dir.join("noise.zshader"),
        r#"
kind = "include"
version = 2
wgsl_files = ["noise.wgsl"]
"#,
    )
    .unwrap();
    fs::write(
        shader_dir.join("noise.wgsl"),
        r#"
fn shader_noise_value() -> f32 {
    return 1.0;
}
"#,
    )
    .unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();

    match manager.load_artifact(&shader_uri).unwrap() {
        ImportedAsset::Shader(shader) => {
            assert_eq!(shader.kind, ShaderAssetKind::Include);
            assert_eq!(
                shader.import_path.as_deref(),
                Some("shader_package_sandbox::noise")
            );
            assert!(shader.validation_diagnostics.is_empty());
        }
        other => panic!("unexpected compound shader artifact: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_reports_redundant_explicit_shader_import_path() {
    let root = unique_temp_project_root("project_manager_shader_import_path_redundant");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Shader Warning Sandbox",
        AssetUri::parse("res://shaders/cloth/common").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let shader_uri = AssetUri::parse("res://shaders/cloth/common").unwrap();
    let shader_meta_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join("cloth")
        .join("common.zmeta");
    let mut shader_meta =
        AssetMetaDocument::new(AssetUuid::new(), shader_uri.clone(), AssetKind::Shader);
    shader_meta.unit = AssetSourceUnit::Compound;
    fs::create_dir_all(shader_meta_path.parent().unwrap()).unwrap();
    shader_meta.save(&shader_meta_path).unwrap();

    let shader_dir = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join("cloth")
        .join("common");
    fs::create_dir_all(&shader_dir).unwrap();
    fs::write(
        shader_dir.join("common.zshader"),
        r#"
kind = "surface"
version = 2
shading_model = "standard_pbr"
import_path = "shader_warning_sandbox::cloth::common"
wgsl_files = ["common.wgsl"]
"#,
    )
    .unwrap();
    fs::write(
        shader_dir.join("common.wgsl"),
        r#"
fn shader_common_value() -> f32 {
    return 1.0;
}
"#,
    )
    .unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();

    match manager.load_artifact(&shader_uri).unwrap() {
        ImportedAsset::Shader(shader) => {
            assert_eq!(shader.kind, ShaderAssetKind::Surface);
            assert_eq!(
                shader.import_path.as_deref(),
                Some("shader_warning_sandbox::cloth::common")
            );
            assert!(shader.validation_diagnostics.is_empty());
        }
        other => panic!("unexpected compound shader artifact: {other:?}"),
    }
    let record = manager
        .registry()
        .get_by_locator(&shader_uri)
        .expect("shader record");
    assert!(record.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("duplicates the derived shader import path")));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_reports_duplicate_shader_import_path_conflicts() {
    let root = unique_temp_project_root("project_manager_shader_import_path_conflict");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Shader Conflict Sandbox",
        AssetUri::parse("res://shaders/a").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let first_uri = write_include_shader_package(&paths, "a", "conflict::shared");
    let second_uri = write_include_shader_package(&paths, "b", "conflict::shared");

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();

    assert!(manager
        .registry()
        .get_by_locator(&first_uri)
        .expect("first shader record")
        .diagnostics
        .is_empty());
    let second_record = manager
        .registry()
        .get_by_locator(&second_uri)
        .expect("second shader record");
    assert!(second_record.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("import_path `conflict::shared` conflicts")
    }));

    let _ = fs::remove_dir_all(root);
}

fn write_include_shader_package(paths: &ProjectPaths, name: &str, import_path: &str) -> AssetUri {
    let shader_uri = AssetUri::parse(&format!("res://shaders/{name}")).unwrap();
    let shader_meta_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join(format!("{name}.zmeta"));
    let mut shader_meta =
        AssetMetaDocument::new(AssetUuid::new(), shader_uri.clone(), AssetKind::Shader);
    shader_meta.unit = AssetSourceUnit::Compound;
    fs::create_dir_all(shader_meta_path.parent().unwrap()).unwrap();
    shader_meta.save(&shader_meta_path).unwrap();

    let shader_dir = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join(name);
    fs::create_dir_all(&shader_dir).unwrap();
    fs::write(
        shader_dir.join(format!("{name}.zshader")),
        format!(
            r#"
kind = "include"
version = 2
import_path = "{import_path}"
wgsl_files = ["{name}.wgsl"]
"#
        ),
    )
    .unwrap();
    fs::write(
        shader_dir.join(format!("{name}.wgsl")),
        format!(
            r#"
fn shader_{name}_value() -> f32 {{
    return 1.0;
}}
"#
        ),
    )
    .unwrap();
    shader_uri
}
