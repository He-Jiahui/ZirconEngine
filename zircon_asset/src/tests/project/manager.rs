use std::fs;

use crate::tests::project::unique_temp_project_root;
use crate::tests::support::{
    write_checker_png, write_default_material, write_default_scene, write_triangle_obj,
    write_valid_wgsl,
};
use crate::{AssetMetaDocument, AssetUri, ImportedAsset, ProjectManager, ResourceId};

#[test]
fn project_manager_scans_assets_imports_library_and_loads_artifacts() {
    let root = unique_temp_project_root("project_manager");
    let paths = crate::ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    crate::ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_default_material(
        paths
            .assets_root()
            .join("materials")
            .join("grid.material.toml"),
    );
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let mut manager = ProjectManager::open(&root).unwrap();
    let imported = manager.scan_and_import().unwrap();

    assert_eq!(manager.manifest().name, "Sandbox");
    assert!(imported.len() >= 5);
    assert!(manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://models/triangle.obj").unwrap())
        .is_some());
    assert!(manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://materials/grid.material.toml").unwrap())
        .is_some());

    let model = manager
        .load_artifact(&AssetUri::parse("res://models/triangle.obj").unwrap())
        .unwrap();
    match model {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    assert!(paths.library_root().join("models").is_dir());
    assert!(paths.library_root().join("materials").is_dir());

    let model_meta_path = paths
        .assets_root()
        .join("models")
        .join("triangle.obj.meta.toml");
    assert!(model_meta_path.exists(), "expected sidecar meta file to be generated");
    let model_meta = AssetMetaDocument::load(&model_meta_path).unwrap();
    let model_record = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://models/triangle.obj").unwrap())
        .unwrap();
    assert_eq!(model_meta.primary_locator, AssetUri::parse("res://models/triangle.obj").unwrap());
    assert_eq!(
        model_record.id(),
        ResourceId::from_asset_uuid_label(model_meta.asset_uuid, None)
    );

    let _ = fs::remove_dir_all(root);
}
