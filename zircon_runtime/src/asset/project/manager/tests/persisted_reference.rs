use std::fs;
use std::path::Path;

use super::*;
use crate::asset::project::{AssetMetaDocument, ProjectManifest, ProjectPaths};
use crate::asset::{AssetKind, AssetUri, AssetUuid};

#[test]
fn writer_rejects_source_replaced_by_link_after_registry_load() {
    let root = std::env::temp_dir().join(format!(
        "zircon_persist_reference_link_{}",
        std::process::id()
    ));
    let outside = root.with_extension("outside");
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&outside);
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "Writer link guard",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let source = root.join("assets/models/hero.glb");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, b"inside").unwrap();
    let guid: AssetUuid = "d1111111-2222-4333-8444-555555555555".parse().unwrap();
    let mut meta = AssetMetaDocument::new(
        guid,
        AssetUri::parse("res://models/hero.glb").unwrap(),
        AssetKind::Model,
    );
    meta.source_digest = "digest".to_owned();
    meta.save(source.with_file_name("hero.glb.zmeta")).unwrap();
    let manager = ProjectManager::open(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let external = outside.join("hero.glb");
    fs::write(&external, b"outside").unwrap();
    fs::remove_file(&source).unwrap();
    if let Err(error) = create_file_link(&external, &source) {
        if error.kind() == std::io::ErrorKind::PermissionDenied {
            let _ = fs::remove_dir_all(&root);
            let _ = fs::remove_dir_all(&outside);
            return;
        }
        panic!("failed to create test link: {error}");
    }

    let error = manager
        .persist_runtime_reference(&AssetReference::new(
            guid,
            AssetUri::parse("res://models/hero.glb").unwrap(),
        ))
        .unwrap_err();
    assert!(matches!(
        error,
        ReferenceResolutionError::MissingPath { .. }
    ));
    assert_eq!(fs::read(&external).unwrap(), b"outside");
    let _ = fs::remove_file(&source);
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&outside);
}

#[cfg(unix)]
fn create_file_link(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_file_link(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}
