use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{AssetRegistryError, AssetRegistryIndex, REGISTRY_FILE_NAME};

#[test]
fn unreadable_registry_does_not_rebuild_from_project_sources() {
    let root = fixture_root("unreadable");
    let registry_root = root.join("registry");
    let formal = registry_root.join(REGISTRY_FILE_NAME);
    fs::create_dir_all(&formal).unwrap();

    // A rebuild would try to scan this regular file as a directory and return its path instead.
    let invalid_asset_root = root.join("invalid-asset-root");
    fs::write(&invalid_asset_root, b"not a directory").unwrap();
    let error = AssetRegistryIndex::load_or_rebuild(&[invalid_asset_root], &registry_root)
        .expect_err("an existing unreadable registry must not trigger a source scan");

    assert!(matches!(
        error,
        AssetRegistryError::Io { path, .. } if path == formal
    ));
    assert!(formal.is_dir());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_registry_still_rebuilds_from_project_sources() {
    let root = fixture_root("missing");
    let registry_root = root.join("registry");
    let index = AssetRegistryIndex::load_or_rebuild(&[], &registry_root).unwrap();

    assert_eq!(index.len(), 0);
    assert!(registry_root.join(REGISTRY_FILE_NAME).is_file());
    fs::remove_dir_all(root).unwrap();
}

fn fixture_root(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon_registry_load_disposition_{label}_{}_{}",
        std::process::id(),
        nonce
    ));
    fs::create_dir_all(&root).unwrap();
    root
}
