use std::fs;
use std::path::Path;

use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::tests::project::binary_artifact_cache_assertions::{
    assert_artifact_cache_files_are_zassets, assert_binary_artifact_cache,
};
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::tests::support::{
    write_checker_png, write_default_material, write_default_scene, write_triangle_obj,
    write_valid_wgsl,
};
use crate::asset::{AssetKind, AssetUri};
use crate::core::resource::ResourceState;

#[test]
fn project_manager_writes_binary_cache_for_render_asset_families() {
    let root = unique_temp_project_root("project_manager_binary_render_cache");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "BinaryRenderCache",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("shaders")
            .join("pbr.wgsl"),
    );
    write_checker_png(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("textures")
            .join("checker.png"),
    );
    write_triangle_obj(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("models")
            .join("triangle.obj"),
    );
    write_default_material(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("materials")
            .join("grid.zmaterial"),
    );
    write_default_scene(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("scenes")
            .join("main.scene.toml"),
    );

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    manager.scan_and_import().unwrap();

    for (locator, kind) in [
        ("res://models/triangle.obj", AssetKind::Model),
        (
            "res://models/triangle.obj#Mesh0/Primitive0",
            AssetKind::Mesh,
        ),
        ("res://scenes/main.scene.toml", AssetKind::Scene),
        ("res://shaders/pbr.wgsl", AssetKind::Shader),
        ("res://materials/grid.zmaterial", AssetKind::Material),
        ("res://textures/checker.png", AssetKind::Texture),
    ] {
        assert_binary_ready_record(&manager, paths.asset_artifact_root(), locator, kind);
    }
    assert_artifact_cache_files_are_zassets(paths.asset_artifact_root());

    let _ = fs::remove_dir_all(root);
}

fn assert_binary_ready_record(
    manager: &ProjectManager,
    artifact_cache_root: &Path,
    locator: &str,
    kind: AssetKind,
) {
    let uri = AssetUri::parse(locator).unwrap();
    let record = manager
        .registry()
        .get_by_locator(&uri)
        .unwrap_or_else(|| panic!("missing ready record for {locator}"));
    assert_eq!(record.kind, kind);
    assert_eq!(record.state, ResourceState::Ready);
    assert_binary_artifact_cache(
        artifact_cache_root,
        record
            .artifact_locator()
            .unwrap_or_else(|| panic!("missing artifact locator for {locator}")),
    );
}
