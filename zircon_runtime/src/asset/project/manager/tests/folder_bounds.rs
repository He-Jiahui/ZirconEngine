use super::*;
use crate::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime_interface::project::RelPath;

fn root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zircon_u09_{label}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn folder_tree_limit_counts_files_and_directories_across_nested_levels() {
    let root = root("tree_inventory_bound");
    fs::create_dir_all(root.join("source/nested")).unwrap();
    fs::write(root.join("source/first.raw"), b"keep").unwrap();
    fs::write(root.join("source/nested/second.raw"), b"keep").unwrap();
    let source = root.join("source");
    assert!(inspect_tree_with_limit(&root, &source, 4).is_ok());
    assert!(inspect_tree_with_limit(&root, &source, 3).is_err());
    assert_eq!(fs::read(source.join("nested/second.raw")).unwrap(), b"keep");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn folder_inventory_limit_is_global_across_registered_roots() {
    let root = root("multi_root_inventory_bound");
    let paths = ProjectPaths::from_root(&root).unwrap();
    let roots = [
        RelPath::project_assets(),
        RelPath::parse("shared-assets").unwrap(),
    ];
    paths.ensure_layout(&roots).unwrap();
    let mut manifest = ProjectManifest::new(
        "Folder inventory bound",
        AssetUri::parse("res://default.scene.toml").unwrap(),
        1,
    );
    manifest.asset_roots = roots.to_vec();
    manifest.save(paths.manifest_path()).unwrap();
    fs::create_dir(root.join("assets/first")).unwrap();
    fs::create_dir(root.join("assets/second")).unwrap();
    fs::create_dir(root.join("shared-assets/third")).unwrap();
    let project = ProjectManager::open(&root).unwrap();
    assert_eq!(project.project_folder_uris_with_limit(3).unwrap().len(), 3);
    assert!(project.project_folder_uris_with_limit(2).is_err());
    fs::remove_dir_all(root).unwrap();
}
