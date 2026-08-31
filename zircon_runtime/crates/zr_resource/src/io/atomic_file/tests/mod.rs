use std::fs;
use std::path::Path;
use std::sync::{Arc, Barrier};

use super::{
    AtomicWriteFault, PathEntry, atomic_write_new, atomic_write_with_fault, classify_path_metadata,
    is_atomic_write_transaction_path, publish_staged_file_for_transaction,
    recover_missing_target_from_backup, stage_atomic_write,
};

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
fn atomic_file_presence_treats_only_not_found_as_missing() {
    let missing = classify_path_metadata(Err(std::io::Error::from(std::io::ErrorKind::NotFound)))
        .expect("NotFound is the only missing-path classification");
    assert_eq!(missing, PathEntry::Missing);

    let error = classify_path_metadata(Err(std::io::Error::from(
        std::io::ErrorKind::PermissionDenied,
    )))
    .expect_err("metadata access failures must not become missing-path evidence");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
}

#[test]
fn backup_create_new_never_overwrites_existing_evidence() {
    let root = test_root("zircon_atomic_backup_create_new");
    fs::create_dir_all(&root).unwrap();
    let source = root.join("config.json");
    let backup = root.join(".config.json.zr-backup-1-1");
    fs::write(&source, b"live generation").unwrap();
    fs::write(&backup, b"existing recovery evidence").unwrap();

    let error = super::transaction::create_backup_file_new(&source, &backup)
        .expect_err("backup fallback must never replace existing evidence");

    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(fs::read(&backup).unwrap(), b"existing recovery evidence");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn concurrent_expected_missing_transaction_publication_has_exactly_one_winner() {
    let root = test_root("zircon_staged_missing_concurrent_publish");
    fs::create_dir_all(&root).unwrap();
    for round in 0..32 {
        let target = Arc::new(root.join(format!("generation-{round}.zmeta")));
        let barrier = Arc::new(Barrier::new(2));
        let workers = [b"first".as_slice(), b"second".as_slice()].map(|bytes| {
            let target = Arc::clone(&target);
            let barrier = Arc::clone(&barrier);
            let staging = root.join(format!(
                "generation-{}.stage",
                crate::io::next_test_output_id()
            ));
            fs::write(&staging, bytes).unwrap();
            std::thread::spawn(move || {
                barrier.wait();
                publish_staged_file_for_transaction(&staging, target.as_path(), false)
                    .map_err(|error| error.into_io_error())
            })
        });

        let outcomes = workers.map(|worker| worker.join().unwrap());
        assert_eq!(
            outcomes.iter().filter(|outcome| outcome.is_ok()).count(),
            1,
            "round {round} must have exactly one publisher"
        );
        assert_eq!(
            outcomes
                .iter()
                .filter_map(|outcome| outcome.as_ref().err())
                .filter(|error| error.kind() == std::io::ErrorKind::AlreadyExists)
                .count(),
            1,
            "round {round} must reject exactly one publisher"
        );
        let published = fs::read(target.as_path()).unwrap();
        assert!(published == b"first" || published == b"second");
    }
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn recovery_ignores_unreserved_backup_lookalikes() {
    let root = test_root("zircon_atomic_recovery_lookalike");
    fs::create_dir_all(&root).unwrap();
    let target = root.join("config.json");
    let lookalike = root.join(".config.json.zr-backup-user-copy");
    fs::write(&lookalike, b"unowned bytes").unwrap();

    assert!(!recover_missing_target_from_backup(&target).unwrap());

    assert!(!target.exists());
    assert_eq!(fs::read(&lookalike).unwrap(), b"unowned bytes");
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn recovery_rejects_non_file_reserved_backup() {
    let root = test_root("zircon_atomic_recovery_non_file");
    fs::create_dir_all(&root).unwrap();
    let target = root.join("config.json");
    let backup = root.join(".config.json.zr-backup-123-4");
    fs::create_dir(&backup).unwrap();

    let error = recover_missing_target_from_backup(&target)
        .expect_err("recovery must not publish a non-file backup candidate");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(!target.exists());
    assert!(backup.is_dir());
    fs::remove_dir_all(root).unwrap();
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
fn atomic_write_new_never_replaces_an_existing_target() {
    let root = test_root("zircon_atomic_new_no_replace");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("copy.zui");
    fs::write(&path, "existing").unwrap();

    let error = atomic_write_new(&path, b"replacement").unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(fs::read_to_string(&path).unwrap(), "existing");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn concurrent_atomic_write_new_publication_has_exactly_one_winner() {
    let root = test_root("zircon_atomic_new_concurrent_publish");
    fs::create_dir_all(&root).unwrap();
    let path = Arc::new(root.join("copy.zui"));
    let staged = Arc::new(Barrier::new(2));
    let workers = [b"first".as_slice(), b"second".as_slice()].map(|payload| {
        let path = Arc::clone(&path);
        let staged = Arc::clone(&staged);
        std::thread::spawn(move || {
            let pending = stage_atomic_write(path.as_path(), payload).unwrap();
            staged.wait();
            pending.commit_new()
        })
    });

    let outcomes = workers.map(|worker| worker.join().unwrap());
    assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
    assert_eq!(
        outcomes
            .iter()
            .filter_map(|outcome| outcome.as_ref().err())
            .filter(|error| error.kind() == std::io::ErrorKind::AlreadyExists)
            .count(),
        1
    );
    let published = fs::read(path.as_path()).unwrap();
    assert!(published == b"first" || published == b"second");
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
    let output_root = std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
        .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .expect("resolve current workspace for atomic-file test output")
                .join("target")
        });
    output_root.join("zircon-test-output").join(format!(
        "{name}_{}_{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ))
}
