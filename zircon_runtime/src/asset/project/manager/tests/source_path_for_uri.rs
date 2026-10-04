use super::ProjectManager;
use crate::asset::project::{ProjectManifest, ProjectPaths};
use crate::asset::AssetUri;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use zircon_runtime_interface::project::RelPath;

#[test]
#[ignore = "requires a Windows reparse-point mutation boundary"]
fn existing_project_source_rechecks_reparse_target_before_world_save() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after the epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("zircon_source_reparse_{nonce}"));
    let outside = std::env::temp_dir().join(format!("zircon_source_reparse_outside_{nonce}"));
    let paths = ProjectPaths::from_root(&root).expect("resolve test project root");
    paths
        .ensure_layout(&[RelPath::project_assets()])
        .expect("create project layout");
    ProjectManifest::new(
        "SourceReparseBoundary",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .expect("write project manifest");
    let project = ProjectManager::open(&root).expect("open project before mutation");
    let asset_root = paths.asset_root(&RelPath::project_assets());
    fs::create_dir_all(&outside).expect("create outside target");
    fs::remove_dir_all(&asset_root).expect("remove original asset root");
    std::os::windows::fs::symlink_dir(&outside, &asset_root).expect("create test reparse point");

    let uri = AssetUri::parse("res://scenes/escape.scene.toml").unwrap();
    let error = project
        .resolve_existing_or_primary_project_source_path_for_uri(&uri)
        .expect_err("reparse target outside the registered root must be rejected");
    assert!(error.to_string().contains("outside registered asset roots"));

    drop(project);
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&outside);
}
