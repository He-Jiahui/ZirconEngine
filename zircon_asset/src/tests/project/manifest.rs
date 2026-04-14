use std::fs;

use crate::{AssetUri, ProjectManifest, ProjectPaths};

use super::unique_temp_project_root;

#[test]
fn project_manifest_roundtrip_preserves_default_scene_and_paths() {
    let root = unique_temp_project_root("manifest");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();

    let manifest = ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        3,
    );
    manifest.save(paths.manifest_path()).unwrap();

    let loaded = ProjectManifest::load(paths.manifest_path()).unwrap();

    assert_eq!(loaded, manifest);
    assert!(paths.assets_root().is_dir());
    assert!(paths.library_root().is_dir());

    let _ = fs::remove_dir_all(root);
}
