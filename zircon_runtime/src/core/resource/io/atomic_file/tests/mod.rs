use std::fs;
use std::path::Path;

use super::{atomic_write_with_fault, is_atomic_write_transaction_path, AtomicWriteFault};

#[test]
fn atomic_transaction_path_recognizes_only_reserved_numeric_siblings() {
    assert!(is_atomic_write_transaction_path(Path::new(
        ".hero.zmeta.zr-staging-123-4"
    )));
    assert!(is_atomic_write_transaction_path(Path::new(
        ".hero.zmeta.zr-backup-123-5"
    )));
    assert!(!is_atomic_write_transaction_path(Path::new(
        ".zr-staging-guide.txt"
    )));
    assert!(!is_atomic_write_transaction_path(Path::new(
        "hero.zmeta.zr-staging-123-4"
    )));
    assert!(!is_atomic_write_transaction_path(Path::new(
        ".hero.zmeta.zr-staging-user-copy"
    )));
}

#[test]
fn atomic_write_replaces_existing_file_and_cleans_transaction_files() {
    let root = test_root("zircon_meta_atomic_commit");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("hero.png.zmeta");
    fs::write(&path, "original").unwrap();

    atomic_write_with_fault(&path, b"replacement", AtomicWriteFault::None).unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "replacement");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn atomic_write_commit_failure_keeps_original_visible_and_cleans_staging_files() {
    let root = test_root("zircon_meta_atomic_restore");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("hero.png.zmeta");
    fs::write(&path, "original").unwrap();

    let error =
        atomic_write_with_fault(&path, b"replacement", AtomicWriteFault::Replace).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::Other);
    assert!(path.is_file());
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn windows_replace_failure_after_backup_restores_the_canonical_target() {
    let root = test_root("zircon_atomic_windows_restore");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("config.json");
    fs::write(&path, "original").unwrap();

    let error =
        atomic_write_with_fault(&path, b"replacement", AtomicWriteFault::ReplaceAfterBackup)
            .unwrap_err();

    assert!(error.to_string().contains("restored from backup"));
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn atomic_write_backup_sync_failure_cleans_staging_and_backup() {
    let root = test_root("zircon_meta_atomic_backup_sync");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("hero.png.zmeta");
    fs::write(&path, "original").unwrap();

    let error =
        atomic_write_with_fault(&path, b"replacement", AtomicWriteFault::BackupSync).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::Other);
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn atomic_write_new_target_reports_parent_sync_failure_after_publication() {
    let root = test_root("zircon_atomic_new_parent_sync");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("preferences-v1/config.json");

    let error =
        atomic_write_with_fault(&path, b"published", AtomicWriteFault::ParentSync).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::Other);
    assert_eq!(fs::read_to_string(&path).unwrap(), "published");
    assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn atomic_write_new_nested_parent_requires_a_directory_durability_barrier() {
    let root = test_root("zircon_atomic_created_directory_sync");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("preferences-v1/namespace/config.zrpref");

    let error = atomic_write_with_fault(
        &path,
        b"preferences",
        AtomicWriteFault::CreatedDirectorySync,
    )
    .unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::Other);
    assert!(!root.join("preferences-v1").exists());
    assert!(!path.exists());

    atomic_write_with_fault(&path, b"preferences", AtomicWriteFault::None).unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"preferences");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn concurrent_directory_is_not_deleted_when_its_durability_barrier_fails() {
    let root = test_root("zircon_atomic_concurrent_directory_sync");
    fs::create_dir_all(&root).unwrap();

    let error = super::sync_new_directory(&root, false, AtomicWriteFault::CreatedDirectorySync)
        .unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::Other);
    assert!(root.is_dir());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn atomic_write_replacement_retains_backup_when_parent_sync_fails() {
    let root = test_root("zircon_atomic_replace_parent_sync");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("config.json");
    fs::write(&path, "original").unwrap();

    let error =
        atomic_write_with_fault(&path, b"replacement", AtomicWriteFault::ParentSync).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::Other);
    assert_eq!(fs::read_to_string(&path).unwrap(), "replacement");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn windows_replacement_retains_backup_when_committed_file_sync_fails() {
    let root = test_root("zircon_atomic_committed_sync");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("config.json");
    fs::write(&path, "original").unwrap();

    let error = atomic_write_with_fault(&path, b"replacement", AtomicWriteFault::CommittedSync)
        .unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::Other);
    assert_eq!(fs::read_to_string(&path).unwrap(), "replacement");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
    fs::remove_dir_all(root).unwrap();
}

fn test_root(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "{name}_{}_{}",
        std::process::id(),
        super::NEXT_ATOMIC_FILE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ))
}
