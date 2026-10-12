//! The real source-sync owner must publish a new activation even with an empty catalog delta.
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::{DefaultEditorAssetManager, ProjectManager};
use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::asset::AssetUri;
use zircon_runtime_interface::project::RelPath;

struct Fixture {
    parent: PathBuf,
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let executable = fs::canonicalize(std::env::current_exe().unwrap()).unwrap();
        let intended = executable
            .parent()
            .unwrap()
            .join("zircon-editor-import-scope-fixtures");
        fs::create_dir_all(&intended).unwrap();
        let parent = fs::canonicalize(&intended).unwrap();
        assert_eq!(parent, intended);
        for _ in 0..128 {
            let root = parent.join(format!(
                "scope-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&root) {
                Ok(()) => {
                    let fixture = Self { parent, root };
                    assert_eq!(fs::canonicalize(&fixture.root).unwrap(), fixture.root);
                    let paths = ProjectPaths::from_root(&fixture.root).unwrap();
                    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
                    ProjectManifest::new(
                        "Unchanged Activation",
                        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
                        1,
                    )
                    .save(paths.manifest_path())
                    .unwrap();
                    return fixture;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create fixture: {error}"),
            }
        }
        panic!("exclusive fixture root unavailable");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if self.root.parent() == Some(self.parent.as_path())
            && fs::canonicalize(&self.parent).ok().as_ref() == Some(&self.parent)
            && fs::canonicalize(&self.root).ok().as_ref() == Some(&self.root)
            && fs::symlink_metadata(&self.root).is_ok_and(|m| !m.file_type().is_symlink())
        {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

#[test]
fn same_root_unchanged_catalog_installs_new_activation_scope() {
    let fixture = Fixture::new();
    let original = ProjectManager::open(&fixture.root).unwrap();
    let manager = DefaultEditorAssetManager::new();
    manager.sync_from_project(original.clone()).unwrap();
    let first_catalog = manager.catalog_snapshot_record();
    let index = manager
        .read_state_recovering_poison()
        .asset_index
        .clone()
        .unwrap();
    assert_eq!(
        index.lock().unwrap().project_activation_identity(),
        Some(original.activation_identity())
    );
    manager.sync_from_project(original.clone()).unwrap();
    assert!(Arc::ptr_eq(
        &first_catalog,
        &manager.catalog_snapshot_record()
    ));
    let reopened = ProjectManager::open(&fixture.root).unwrap();
    assert_ne!(
        reopened.activation_identity(),
        original.activation_identity()
    );
    assert!(reopened
        .catalog_input_generation()
        .delta_since(&original.catalog_input_generation())
        .is_unchanged());
    manager.sync_from_project(reopened.clone()).unwrap();
    let after = manager.catalog_snapshot_record();
    assert!(!Arc::ptr_eq(&first_catalog, &after));
    assert!(after.catalog_revision > first_catalog.catalog_revision);
    assert!(Arc::ptr_eq(
        &index,
        &manager
            .read_state_recovering_poison()
            .asset_index
            .clone()
            .unwrap()
    ));
    assert_eq!(
        index.lock().unwrap().project_activation_identity(),
        Some(reopened.activation_identity())
    );
}
