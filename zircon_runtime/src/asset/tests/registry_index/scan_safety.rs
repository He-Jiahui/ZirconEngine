use crate::asset::registry::{AssetRegistryError, AssetRegistryIndex};
use crate::asset::{AssetKind, AssetUuid};

use super::{registry_root, unique_root, write_asset};

#[test]
fn metadata_scan_rejects_link_or_reparse_directory_before_following_it() {
    let project = unique_root("registry_scan_link");
    let assets = project.join("assets");
    let outside = unique_root("registry_scan_outside");
    std::fs::create_dir_all(&assets).unwrap();
    write_asset(
        &outside,
        "escaped.data",
        AssetUuid::new(),
        AssetKind::Data,
        vec![],
    );
    let linked = assets.join("linked");
    if !create_directory_link(&outside, &linked) {
        let _ = std::fs::remove_dir_all(project);
        let _ = std::fs::remove_dir_all(outside);
        return;
    }

    let error =
        AssetRegistryIndex::rebuild_from_project(&[assets], registry_root(&project)).unwrap_err();

    assert!(matches!(
        error,
        AssetRegistryError::UnsafeMetadataLink { path, .. } if path == linked
    ));
    let _ = std::fs::remove_dir_all(project);
    let _ = std::fs::remove_dir_all(outside);
}

#[cfg(unix)]
fn create_directory_link(target: &std::path::Path, link: &std::path::Path) -> bool {
    std::os::unix::fs::symlink(target, link).is_ok()
}

#[cfg(windows)]
fn create_directory_link(target: &std::path::Path, link: &std::path::Path) -> bool {
    match std::os::windows::fs::symlink_dir(target, link) {
        Ok(()) => true,
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => false,
        Err(error) => panic!("create directory reparse fixture failed: {error}"),
    }
}
