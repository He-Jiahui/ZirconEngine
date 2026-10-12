use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::{
    AssetMetaDocument, AssetSourceUnit, ProjectManifest, ProjectPaths,
};
use zircon_runtime::asset::{AssetKind, AssetUri, AssetUuid};
use zircon_runtime::plugin::PluginPackageManifest;

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::core::resource::ResourceState;

use super::super::default_editor_asset_manager::DefaultEditorAssetManager;

#[test]
fn sync_from_project_recovers_from_a_poisoned_state_lock() {
    let root = unique_temp_project_root("sync_poisoned_state_lock");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Poisoned State Lock",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let manager = DefaultEditorAssetManager::new();
    let state = Arc::clone(&manager.state);
    let poisoner = std::thread::spawn(move || {
        let _guard = state.write().expect("state write lock");
        panic!("poison the editor asset state lock");
    });
    assert!(poisoner.join().is_err());

    assert!(manager
        .sync_from_project(ProjectManager::open(&root).unwrap())
        .is_ok());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sync_from_project_does_not_republish_an_unchanged_generation() {
    let root = unique_temp_project_root("sync_unchanged_generation");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "UnchangedProject",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let material_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("materials")
        .join("unchanged.material.toml");
    fs::create_dir_all(material_path.parent().unwrap()).unwrap();
    fs::write(&material_path, "not valid toml = [").unwrap();

    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let manager = DefaultEditorAssetManager::new();
    manager.sync_from_project(project.clone()).unwrap();
    let first = manager.catalog_snapshot_record();

    manager.sync_from_project(project).unwrap();
    let second = manager.catalog_snapshot_record();

    assert!(std::sync::Arc::ptr_eq(&first, &second));
    assert_eq!(first.catalog_revision, second.catalog_revision);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sync_from_project_keeps_error_assets_without_artifacts_in_catalog() {
    let root = unique_temp_project_root("sync_error_asset_without_artifact");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "BrokenAssetProject",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let material_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("materials")
        .join("broken.material.toml");
    fs::create_dir_all(material_path.parent().unwrap()).unwrap();
    fs::write(&material_path, "not valid toml = [").unwrap();

    let mut project = ProjectManager::open(&root).unwrap();
    let records = project.scan_and_import().unwrap();
    assert!(records
        .iter()
        .any(|record| record.state == ResourceState::Error && record.artifact_locator.is_none()));

    let manager = DefaultEditorAssetManager::new();
    manager.sync_from_project(project).unwrap();
    let catalog = manager.catalog_snapshot_record();
    let broken = catalog
        .assets
        .iter()
        .find(|asset| asset.locator == "res://materials/broken.material.toml")
        .expect("broken material remains visible in editor catalog");
    assert!(!broken.diagnostics.is_empty());
    assert!(broken.direct_reference_uuids.is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sync_from_project_exposes_zmeta_package_and_compound_shader_details() {
    let root = unique_temp_project_root("sync_zmeta_compound_shader");
    let package_root = unique_temp_project_root("sync_zmeta_package");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "ZMetaEditorProject",
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
wgsl_files = ["unlit.wgsl"]
"#,
    )
    .unwrap();
    fs::write(
        shader_dir.join("unlit.wgsl"),
        r#"
fn zr_material_surface(input: ZrSurfaceInput) -> ZrSurfaceOutput {
var surface = zr_surface_default(input);
surface.base_color = vec4f(1.0, 1.0, 1.0, 1.0);
return surface;
}
"#,
    )
    .unwrap();

    let package_asset_path = package_root.join("assets").join("nav").join("agent.json");
    fs::create_dir_all(package_asset_path.parent().unwrap()).unwrap();
    fs::write(&package_asset_path, r#"{ "agent": true }"#).unwrap();
    let package_manifest = PluginPackageManifest::new("navigation", "Navigation")
        .with_package_identity("com", "zircon", "navigation");

    let mut project = ProjectManager::open(&root).unwrap();
    project
        .register_package_asset_roots(
            package_manifest.package_id(),
            package_manifest.asset_roots_or_default(),
            &package_root,
        )
        .unwrap();
    project.scan_and_import().unwrap();

    let manager = DefaultEditorAssetManager::new();
    manager.sync_from_project(project).unwrap();

    let catalog = manager.catalog_snapshot_record();
    assert!(catalog
        .folders
        .iter()
        .any(|folder| folder.folder_id == "package://com.zircon.navigation"));
    let shader = catalog
        .assets
        .iter()
        .find(|asset| asset.locator == "res://shaders/unlit_shader")
        .expect("compound shader is visible in editor catalog");
    assert!(
        shader.diagnostics.is_empty(),
        "compound shader fixture must import before editor detail projection: {:?}",
        shader.diagnostics
    );
    let details = manager
        .asset_details_generation(&shader.uuid)
        .expect("shader details");
    assert_eq!(details.unit, AssetSourceUnit::Compound);
    assert!(details.package_id.is_none());
    assert!(details
        .included_files
        .contains(&"res://shaders/unlit_shader/unlit.zshader".to_string()));
    assert!(details
        .included_files
        .contains(&"res://shaders/unlit_shader/unlit.wgsl".to_string()));
    assert!(
        details
            .subassets
            .iter()
            .any(|subasset| subasset.locator.ends_with("#zshader:unlit.zshader")),
        "zshader subasset should be projected from .zmeta entries: {:?}",
        details.subassets
    );
    assert!(details
        .subassets
        .iter()
        .any(|subasset| subasset.locator.ends_with("#wgsl:unlit.wgsl")));

    let package_asset = catalog
        .assets
        .iter()
        .find(|asset| asset.locator == "package://com.zircon.navigation/nav/agent.json")
        .expect("package asset is visible in editor catalog");
    let package_details = manager
        .asset_details_generation(&package_asset.uuid)
        .expect("package details");
    assert_eq!(
        package_details.package_id.as_deref(),
        Some("com.zircon.navigation")
    );
    assert_eq!(package_details.unit, AssetSourceUnit::Single);

    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(package_root);
}

#[test]
fn sync_from_project_refreshes_shader_ide_environment_after_import() {
    let root = unique_temp_project_root("sync_shader_ide_env");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Shader Ide Sandbox",
        AssetUri::parse("res://shaders/hero").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    write_shader_ide_surface_package(&paths);

    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let manager = DefaultEditorAssetManager::new();
    manager.sync_from_project(project).unwrap();

    let shader_uri = AssetUri::parse("res://shaders/hero").unwrap();
    let ide_root = ProjectPaths::from_root(&root)
        .unwrap()
        .cache_root()
        .join(zircon_runtime::core::framework::render::SHADER_IDE_ENV_CACHE_DIR);
    let module_map_path =
        ide_root.join(zircon_runtime::core::framework::render::SHADER_IDE_MODULE_MAP_FILE);
    let preview_path = ide_root.join(
        zircon_runtime::core::framework::render::shader_ide_preview_relative_path(
            &shader_uri,
            zircon_runtime::core::framework::render::SHADER_IDE_PREVIEW_DEFAULT_VARIANT,
        ),
    );
    let segment_path = ide_root.join(
        zircon_runtime::core::framework::render::shader_ide_preview_segments_relative_path(
            &shader_uri,
            zircon_runtime::core::framework::render::SHADER_IDE_PREVIEW_DEFAULT_VARIANT,
        ),
    );

    let module_map = fs::read_to_string(module_map_path).unwrap();
    assert!(module_map.contains("shader_ide_sandbox::hero"));
    assert!(module_map.contains("generated/res_shaders_hero.material.wgsl"));
    assert!(fs::read_to_string(preview_path)
        .unwrap()
        .contains("fn zr_material_surface"));
    assert!(fs::read_to_string(segment_path)
        .unwrap()
        .contains("generated_material"));

    let _ = fs::remove_dir_all(root);
}

fn write_shader_ide_surface_package(paths: &ProjectPaths) {
    let shader_uri = AssetUri::parse("res://shaders/hero").unwrap();
    let shader_meta_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join("hero.zmeta");
    let mut shader_meta = AssetMetaDocument::new(AssetUuid::new(), shader_uri, AssetKind::Shader);
    shader_meta.unit = AssetSourceUnit::Compound;
    fs::create_dir_all(shader_meta_path.parent().unwrap()).unwrap();
    shader_meta.save(&shader_meta_path).unwrap();

    let shader_dir = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join("hero");
    fs::create_dir_all(&shader_dir).unwrap();
    fs::write(
        shader_dir.join("hero.zshader"),
        r#"
kind = "surface"
version = 2
shading_model = "standard_pbr"
wgsl_files = ["hero.wgsl"]

[[properties]]
name = "base_color"
kind = "vec4"
default = [0.8, 0.4, 0.2, 1.0]
"#,
    )
    .unwrap();
    fs::write(
        shader_dir.join("hero.wgsl"),
        r#"
#include <self::material>

fn zr_material_surface(input: ZrSurfaceInput) -> ZrSurfaceOutput {
var surface = zr_surface_default(input);
surface.base_color = zr_mat_base_color();
return surface;
}
"#,
    )
    .unwrap();
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_editor_{label}_{nanos}"))
}
