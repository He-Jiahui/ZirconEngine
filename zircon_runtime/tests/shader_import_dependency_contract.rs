use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::{
    project::{AssetMetaDocument, AssetSourceUnit, ProjectManager, ProjectManifest, ProjectPaths},
    AssetKind, AssetUri, AssetUuid,
};

#[test]
fn project_shader_source_only_imports_become_reload_dependencies() {
    let root = unique_temp_project_root("shader_import_dependency_contract");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Shader Hot Reload Sandbox",
        AssetUri::parse("res://shaders/surface").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let shared_uri =
        write_include_shader_package(&paths, "shared", "shader_hot_reload_sandbox::shared");
    let surface_uri = write_surface_shader_package(
        &paths,
        "surface",
        "shader_hot_reload_sandbox::surface",
        Some("shader_hot_reload_sandbox::shared"),
    );
    let bystander_uri = write_surface_shader_package(
        &paths,
        "bystander",
        "shader_hot_reload_sandbox::bystander",
        None,
    );

    let mut manager = ProjectManager::open(&root).unwrap();
    let imported = manager.scan_and_import().unwrap();
    let registry = manager.registry();
    let shared_record = registry
        .get_by_locator(&shared_uri)
        .expect("shared include shader record");
    let surface_record = registry
        .get_by_locator(&surface_uri)
        .expect("surface shader record");
    let bystander_record = registry
        .get_by_locator(&bystander_uri)
        .expect("bystander shader record");

    assert_eq!(surface_record.dependency_ids, vec![shared_record.id()]);
    assert!(bystander_record.dependency_ids.is_empty());
    assert!(imported.iter().any(|record| {
        record.id() == surface_record.id() && record.dependency_ids == vec![shared_record.id()]
    }));

    let _ = fs::remove_dir_all(root);
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

fn write_include_shader_package(paths: &ProjectPaths, name: &str, import_path: &str) -> AssetUri {
    let shader_uri = AssetUri::parse(&format!("res://shaders/{name}")).unwrap();
    let shader_meta_path = paths
        .assets_root()
        .join("shaders")
        .join(format!("{name}.zmeta"));
    let mut shader_meta =
        AssetMetaDocument::new(AssetUuid::new(), shader_uri.clone(), AssetKind::Shader);
    shader_meta.unit = AssetSourceUnit::Compound;
    fs::create_dir_all(shader_meta_path.parent().unwrap()).unwrap();
    shader_meta.save(&shader_meta_path).unwrap();

    let shader_dir = paths.assets_root().join("shaders").join(name);
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

fn write_surface_shader_package(
    paths: &ProjectPaths,
    name: &str,
    import_path: &str,
    import_source: Option<&str>,
) -> AssetUri {
    let shader_uri = AssetUri::parse(&format!("res://shaders/{name}")).unwrap();
    let shader_meta_path = paths
        .assets_root()
        .join("shaders")
        .join(format!("{name}.zmeta"));
    let mut shader_meta =
        AssetMetaDocument::new(AssetUuid::new(), shader_uri.clone(), AssetKind::Shader);
    shader_meta.unit = AssetSourceUnit::Compound;
    fs::create_dir_all(shader_meta_path.parent().unwrap()).unwrap();
    shader_meta.save(&shader_meta_path).unwrap();

    let shader_dir = paths.assets_root().join("shaders").join(name);
    fs::create_dir_all(&shader_dir).unwrap();
    let import_block = import_source
        .map(|source| {
            format!(
                r#"
[[imports]]
source = "{source}"
"#
            )
        })
        .unwrap_or_default();
    fs::write(
        shader_dir.join(format!("{name}.zshader")),
        format!(
            r#"
kind = "surface"
version = 2
shading_model = "standard_pbr"
import_path = "{import_path}"
wgsl_files = ["{name}.wgsl"]
{import_block}
"#
        ),
    )
    .unwrap();

    let include_line = import_source
        .map(|source| format!("#include <{source}>\n"))
        .unwrap_or_default();
    fs::write(
        shader_dir.join(format!("{name}.wgsl")),
        format!(
            r#"{include_line}
fn shader_{name}_surface_value() -> f32 {{
    return 1.0;
}}
"#
        ),
    )
    .unwrap();
    shader_uri
}
